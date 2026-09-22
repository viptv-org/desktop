//! Desktop-only SmartCast LAN discovery.
//!
//! The shared core defines the candidate contract (`vizio_discovery_candidates`:
//! every host of a caller-approved /24, modern port 7345 then legacy 9000) and
//! the identity rule (`vizio_deviceinfo_name`). This module executes it:
//! every candidate gets a bounded TCP connect, and every answering host is
//! asked for the unauthenticated `/state/device/deviceinfo` document that
//! names real televisions. A bounded SSDP M-SEARCH still runs alongside as a
//! secondary source, but multicast is filtered on many networks, so the
//! direct probe is authoritative. No pairing state or session lock is
//! touched, every path is deadline-bounded, and discovery exposes network
//! names only.

use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

// SSDP discovery: the desktop-only extension of the copied adapter. One
// ephemeral UDP socket owns the whole exchange, no SmartCastState or session
// lock is touched, and every path is deadline-bounded, so the search can
// never wedge pairing, remote, or window commands.

/// Multicast SSDP endpoint every M-SEARCH is sent to; replies are unicast
/// back to the ephemeral socket that asked.
const SSDP_GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
/// Whole-network sweep plus the DIAL service SmartCast TVs announce for
/// casting. SSDP lets a responder drop any single datagram, so both are sent
/// twice, one second apart, inside the bounded window.
const SSDP_SEARCH_ALL: &str = "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nMX: 1\r\nST: ssdp:all\r\n\r\n";
const SSDP_SEARCH_DIAL: &str = "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nMX: 1\r\nST: urn:dial-multiscreen-org:service:dial:1\r\n\r\n";
/// Hard ceiling on collection: MX=1 spreads replies across one second, the
/// resend lands one second in, and the tail absorbs slow responders.
const DISCOVERY_WINDOW: Duration = Duration::from_millis(3_000);
const DISCOVERY_RESEND_AFTER: Duration = Duration::from_millis(1_000);
/// Renderer-visible cap; hosts are deduplicated first, so this counts
/// distinct televisions, not their per-service SSDP replies.
const DISCOVERY_LIMIT: usize = 16;

// Direct probe: the authoritative source. Bounded by construction —
// 508 candidates at 48 in flight and 350 ms each complete in under four
// seconds, and only a port that actually answers is asked for deviceinfo.

/// Per-candidate TCP connect ceiling; also bounds unreachable hosts, whose
/// ARP resolution would otherwise stall for seconds.
const PROBE_CONNECT: Duration = Duration::from_millis(350);
/// Per-host deviceinfo request ceiling once the port answered.
const PROBE_REQUEST: Duration = Duration::from_millis(1_500);
/// Probe fan-out cap so one sweep cannot saturate the LAN interface.
const PROBE_CONCURRENCY: usize = 48;

#[derive(Serialize, Clone)]
pub(crate) struct DiscoveredTv {
    pub(crate) name: String,
    pub(crate) host: String,
}

/// List Vizio SmartCast TVs on the local network as
/// `[{"name":"Vizio TV (192.168.1.50)","host":"192.168.1.50"}]`-shaped JSON
/// text; the renderer parses it and never learns more than network names.
#[tauri::command]
pub async fn smartcast_discover() -> Result<String, String> {
    // The direct probe is authoritative; SSDP failures (multicast filtering
    // is common) must not hide probe results.
    let (probed, ssdp) = tokio::join!(discover_by_probe(), async {
        tokio::task::spawn_blocking(discover_ssdp)
            .await
            .unwrap_or_else(|_| Ok(Vec::new()))
            .unwrap_or_default()
    });
    let discovered = merge_discovered(probed, ssdp);
    serde_json::to_string(&discovered)
        .map_err(|_| "Could not encode the discovered TV list".to_owned())
}

/// Probe every core-listed candidate of the default-route /24. The caller
/// approves exactly one /24 (the interface the OS would route through), the
/// same scope the shared core documents for mobile shells.
async fn discover_by_probe() -> Vec<DiscoveredTv> {
    let Some(prefix) = default_route_prefix() else {
        return Vec::new();
    };
    let client = match reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(_) => return Vec::new(),
    };
    let semaphore = std::sync::Arc::new(Semaphore::new(PROBE_CONCURRENCY));
    let candidates: Vec<String> = (1..=254u32)
        .flat_map(|host| [7345, 9000].map(|port| format!("{prefix}.{host}:{port}")))
        .collect();
    let mut tasks = Vec::with_capacity(candidates.len());
    for address in candidates {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("the discovery semaphore is never closed");
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = permit;
            probe_candidate(&client, &address).await
        }));
    }
    let mut discovered: Vec<DiscoveredTv> = Vec::new();
    for task in tasks {
        if let Ok(Some(entry)) = task.await {
            if !discovered.iter().any(|found| found.host == entry.host) {
                discovered.push(entry);
                if discovered.len() == DISCOVERY_LIMIT {
                    break;
                }
            }
        }
    }
    discovered
}

/// One candidate endpoint: a bounded TCP connect, then the unauthenticated
/// deviceinfo document whose shape the shared core validates.
async fn probe_candidate(client: &reqwest::Client, address: &str) -> Option<DiscoveredTv> {
    let host = address.rsplit_once(':')?.0.to_owned();
    let connected = timeout(PROBE_CONNECT, TcpStream::connect(address)).await.ok()?;
    connected.ok()?;
    let url = format!("https://{address}/state/device/deviceinfo");
    let response = timeout(PROBE_REQUEST, client.get(&url).send()).await.ok()?.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = timeout(PROBE_REQUEST, response.text()).await.ok()?.ok()?;
    let name = viptv_core::vizio_deviceinfo_name(body)?;
    let name = if name.is_empty() { "Vizio TV" } else { &name };
    Some(DiscoveredTv {
        name: format!("{name} ({host})"),
        host,
    })
}

/// The /24 prefix of the interface the OS routes through, derived without
/// emitting a packet. Returns nothing when there is no usable route, which
/// leaves SSDP as the only source.
fn default_route_prefix() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    // UDP connect only programs the routing decision; nothing is sent.
    socket.connect("8.8.8.8:80").ok()?;
    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(ip) => Some(subnet_prefix(&ip)),
        IpAddr::V6(_) => None,
    }
}

fn subnet_prefix(ip: &Ipv4Addr) -> String {
    let octets = ip.octets();
    format!("{}.{}.{}", octets[0], octets[1], octets[2])
}

/// Probe entries win over SSDP for the same host: the probe asked the TV for
/// its own name, while SSDP reports a service string.
fn merge_discovered(probed: Vec<DiscoveredTv>, ssdp: Vec<DiscoveredTv>) -> Vec<DiscoveredTv> {
    let mut merged = probed;
    for entry in ssdp {
        if !merged.iter().any(|found| found.host == entry.host) {
            merged.push(entry);
        }
    }
    merged.truncate(DISCOVERY_LIMIT);
    merged
}

fn discover_ssdp() -> Result<Vec<DiscoveredTv>, String> {
    let endpoint = SocketAddrV4::new(SSDP_GROUP, 1900);
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|_| "Could not open a local network socket to search for TVs".to_owned())?;
    // Membership is a best-effort courtesy for multicast-picky interfaces;
    // M-SEARCH replies arrive as ordinary unicast on this socket.
    let _ = socket.join_multicast_v4(&SSDP_GROUP, &Ipv4Addr::UNSPECIFIED);
    socket
        .set_nonblocking(true)
        .map_err(|_| "Could not prepare the local TV search".to_owned())?;
    send_search(&socket, endpoint)
        .map_err(|_| "Could not send the TV search on the local network".to_owned())?;
    let deadline = std::time::Instant::now() + DISCOVERY_WINDOW;
    let resend_at = std::time::Instant::now() + DISCOVERY_RESEND_AFTER;
    let mut resent = false;
    let mut buffer = [0u8; 4096];
    let mut discovered: Vec<DiscoveredTv> = Vec::new();
    // Non-blocking drain loop: WouldBlock and transient socket errors park
    // ~50ms, the deadline bounds every branch, and nothing here can panic.
    while discovered.len() < DISCOVERY_LIMIT && std::time::Instant::now() < deadline {
        if !resent && std::time::Instant::now() >= resend_at {
            resent = true;
            let _ = send_search(&socket, endpoint);
        }
        match socket.recv_from(&mut buffer) {
            Ok((length, source)) => {
                let Some(entry) = ssdp_response_tv(&buffer[..length], source) else { continue };
                if !discovered.iter().any(|found| found.host == entry.host) {
                    discovered.push(entry);
                }
            }
            Err(_) => std::thread::sleep(Duration::from_millis(50)),
        }
    }
    Ok(discovered)
}

fn send_search(socket: &UdpSocket, endpoint: SocketAddrV4) -> std::io::Result<()> {
    socket.send_to(SSDP_SEARCH_ALL.as_bytes(), endpoint)?;
    socket.send_to(SSDP_SEARCH_DIAL.as_bytes(), endpoint)?;
    Ok(())
}

/// Turn one M-SEARCH reply into a renderer entry: the UDP source address is
/// the TV host and SERVER/USN/LOCATION headers carry the only identity we
/// expose. Responders that never mention Vizio (routers, speakers, other
/// cast targets) are dropped so the list stays connectable SmartCast TVs.
fn ssdp_response_tv(reply: &[u8], source: SocketAddr) -> Option<DiscoveredTv> {
    let host = match source.ip() {
        std::net::IpAddr::V4(ip) => ip.to_string(),
        // SmartCast discovery targets the IPv4 LAN; skip everything else.
        std::net::IpAddr::V6(_) => return None,
    };
    let text = std::str::from_utf8(reply).ok()?;
    if !text.starts_with("HTTP/") {
        return None; // NOTIFY announcements and other multicast noise.
    }
    let server = header_value(text, "SERVER");
    let usn = header_value(text, "USN");
    let location = header_value(text, "LOCATION");
    let branded = [server, usn, location]
        .into_iter()
        .any(|header| header.is_some_and(|value| value.to_ascii_lowercase().contains("vizio")));
    if !branded {
        return None;
    }
    let name = server
        .and_then(vizio_segment)
        .filter(|segment| !segment.is_empty())
        .unwrap_or("Vizio TV");
    Some(DiscoveredTv {
        name: format!("{name} ({host})"),
        host,
    })
}

/// Case-insensitive lookup of one `Name: value` header in an SSDP reply;
/// names are case-insensitive and values may themselves contain colons.
fn header_value<'a>(reply: &'a str, name: &str) -> Option<&'a str> {
    reply.split("\r\n").skip(1).find_map(|line| {
        let (key, value) = line.split_once(':')?;
        key.trim().eq_ignore_ascii_case(name).then(|| value.trim())
    })
}

/// The Vizio-branded tail of a SERVER header, e.g. "VIZIO SmartCast/19.0.4"
/// becomes "VIZIO SmartCast"; the firmware version after the slash is noise.
fn vizio_segment(server: &str) -> Option<&str> {
    let start = server.to_ascii_lowercase().find("vizio")?;
    let rest = &server[start..];
    Some(rest[..rest.find('/').unwrap_or(rest.len())].trim())
}

#[cfg(test)]
mod tests {
    use super::*;

        #[test]
        fn discovery_names_come_from_ssdp_headers_with_ip_fallback() {
            let source: SocketAddr = "192.0.2.50:1900".parse().unwrap();
            let branded = "HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age=1800\r\nSERVER: Linux/2.6.18 UPnP/1.0 VIZIO SmartCast/19.0.4\r\nLOCATION: http://192.0.2.50:4700/device_description.xml\r\nUSN: uuid:5ec5a51e-8cb3-4c0d-8b1e-6d1c5d6b9f1a::urn:dial-multiscreen-org:service:dial:1\r\n\r\n";
            let tv = ssdp_response_tv(branded.as_bytes(), source).unwrap();
            assert_eq!(tv.host, "192.0.2.50");
            assert_eq!(tv.name, "VIZIO SmartCast (192.0.2.50)");
            let unbranded_server = ssdp_response_tv(b"HTTP/1.1 200 OK\r\nUSN: uuid:aa::vizio-smartcast\r\n\r\n", source).unwrap();
            assert_eq!(unbranded_server.name, "Vizio TV (192.0.2.50)");
        }

    #[test]
    fn discovery_ignores_non_vizio_replies_and_multicast_noise() {
        let source: SocketAddr = "192.0.2.51:1900".parse().unwrap();
        assert!(ssdp_response_tv(b"HTTP/1.1 200 OK\r\nSERVER: Linux/3.4 UPnP/1.0\r\n\r\n", source).is_none());
        assert!(ssdp_response_tv(b"NOTIFY * HTTP/1.1\r\nSERVER: VIZIO SmartCast\r\n\r\n", source).is_none());
        assert!(ssdp_response_tv(b"\xff\xfe not text", source).is_none());
        assert!(ssdp_response_tv(b"HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age=1800\r\nLOCATION: http://192.0.2.51:4700/dd.xml\r\n\r\n", source).is_none());
    }

    #[test]
    fn probe_results_win_the_name_for_a_host_ssdp_also_found() {
        let probed = vec![DiscoveredTv {
            name: "living room 65 (192.0.2.23)".into(),
            host: "192.0.2.23".into(),
        }];
        let ssdp = vec![
            DiscoveredTv {
                name: "VIZIO SmartCast (192.0.2.23)".into(),
                host: "192.0.2.23".into(),
            },
            DiscoveredTv {
                name: "VIZIO SmartCast (192.0.2.9)".into(),
                host: "192.0.2.9".into(),
            },
        ];
        let merged = merge_discovered(probed, ssdp);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].name, "living room 65 (192.0.2.23)");
        assert_eq!(merged[1].host, "192.0.2.9");
    }

    #[test]
    fn subnet_prefix_drops_the_host_octet() {
        assert_eq!(subnet_prefix(&"192.168.88.14".parse().unwrap()), "192.168.88");
        assert_eq!(subnet_prefix(&"10.0.0.1".parse().unwrap()), "10.0.0");
    }

    /// Real-network check for this LAN: run with
    /// `cargo test -- --ignored --nocapture` while a SmartCast TV is on.
    #[test]
    #[ignore = "executes a real LAN probe"]
    fn real_lan_probe_finds_a_smartcast_tv() {
        let found = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(discover_by_probe());
        for entry in &found {
            println!("{} {}", entry.name, entry.host);
        }
        assert!(!found.is_empty(), "the probe found no SmartCast TVs");
    }
}
