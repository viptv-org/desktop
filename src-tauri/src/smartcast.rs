//! SmartCast host adapter copied from `core/adapters/tauri/smartcast.rs`
//! (see that repository's README) with one desktop-only addition:
//! `smartcast_discover`, a bounded SSDP M-SEARCH over `std::net` UDP that
//! lists Vizio SmartCast TVs on the local network. All SmartCast traffic
//! stays native; the React renderer never sees TV credentials, TLS policy,
//! or pairing state, and discovery exposes network names only.

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Notify};
use url::Url;
use viptv_core::SmartCastBridge;

const CREDENTIAL_SERVICE: &str = "org.viptv.smartcast";

pub struct SmartCastState {
    session: Mutex<Option<Session>>,
    cancelled: AtomicBool,
    cancel_notify: Notify,
}

impl Default for SmartCastState {
    fn default() -> Self {
        Self {
            session: Mutex::new(None),
            cancelled: AtomicBool::new(false),
            cancel_notify: Notify::new(),
        }
    }
}

struct Session {
    bridge: SmartCastBridge,
    client: reqwest::Client,
    origin: String,
    credential_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlannedRequest {
    method: String,
    url: String,
    headers: BTreeMap<String, String>,
    body: Option<Map<String, Value>>,
    timeout_millis: u64,
    max_response_bytes: u64,
}

#[tauri::command]
pub async fn smartcast_configure(
    state: tauri::State<'_, SmartCastState>,
    host: String,
    device_id: String,
    device_name: String,
    credential_id: String,
) -> Result<(), String> {
    if credential_id.is_empty()
        || credential_id.len() > 128
        || !credential_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err("Invalid SmartCast credential identifier".into());
    }
    let origin = configured_origin(&host)?;
    let credential = keyring::Entry::new(CREDENTIAL_SERVICE, &credential_id)
        .map_err(|_| "SmartCast credential vault is unavailable")?
        .get_password()
        .ok();
    let bridge = SmartCastBridge::new(
        json!({
            "host": origin,
            "authToken": credential,
            "deviceId": device_id,
            "deviceName": device_name,
        })
        .to_string(),
    )
    .map_err(|_| "Invalid SmartCast configuration")?;
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "SmartCast transport is unavailable")?;
    *state.session.lock().await = Some(Session {
        bridge,
        client,
        origin,
        credential_id,
    });
    Ok(())
}

#[tauri::command]
pub async fn smartcast_run(
    state: tauri::State<'_, SmartCastState>,
    operation: String,
    input: String,
) -> Result<String, String> {
    state.cancelled.store(false, Ordering::Release);
    let mut guard = state.session.lock().await;
    let session = guard
        .as_mut()
        .ok_or_else(|| "SmartCast is not configured".to_owned())?;
    let mut output = session
        .bridge
        .start(operation, input)
        .map_err(|_| "SmartCast core is unavailable")?;
    loop {
        let parsed: Value =
            serde_json::from_str(&output).map_err(|_| "SmartCast core returned invalid output")?;
        if parsed.get("kind").and_then(Value::as_str) != Some("request") {
            if parsed
                .get("credentialChanged")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                let token = session
                    .bridge
                    .credential()
                    .map_err(|_| "SmartCast core is unavailable")?
                    .ok_or_else(|| "SmartCast pairing did not return a credential".to_owned())?;
                keyring::Entry::new(CREDENTIAL_SERVICE, &session.credential_id)
                    .map_err(|_| "SmartCast credential vault is unavailable")?
                    .set_password(&token)
                    .map_err(|_| "Could not store SmartCast credential")?;
            }
            return Ok(output);
        }
        let request_id = parsed
            .get("requestId")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| "SmartCast core returned an invalid request ID".to_owned())?;
        let request = serde_json::from_value::<PlannedRequest>(
            parsed.get("request").cloned().unwrap_or(Value::Null),
        )
        .map_err(|_| "SmartCast core returned an invalid request")?;
        output = match execute(session, request, &state.cancelled, &state.cancel_notify).await {
            Ok((status, body)) => session.bridge.resolve(request_id, status, body),
            Err(()) => session.bridge.reject(request_id),
        }
        .map_err(|_| "SmartCast core is unavailable")?;
    }
}

#[tauri::command]
pub fn smartcast_cancel(state: tauri::State<'_, SmartCastState>) {
    state.cancelled.store(true, Ordering::Release);
    state.cancel_notify.notify_one();
}

#[tauri::command]
pub async fn smartcast_forget(state: tauri::State<'_, SmartCastState>) -> Result<(), String> {
    state.cancelled.store(true, Ordering::Release);
    state.cancel_notify.notify_one();
    let mut guard = state.session.lock().await;
    if let Some(session) = guard.as_mut() {
        session
            .bridge
            .clear_credential()
            .map_err(|_| "SmartCast core is unavailable")?;
        match keyring::Entry::new(CREDENTIAL_SERVICE, &session.credential_id)
            .map_err(|_| "SmartCast credential vault is unavailable")?
            .delete_credential()
        {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(_) => return Err("Could not clear SmartCast credential".into()),
        }
    }
    Ok(())
}

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

#[derive(Serialize)]
struct DiscoveredTv {
    name: String,
    host: String,
}

/// List Vizio SmartCast TVs on the local network as
/// `[{"name":"Vizio TV (192.168.1.50)","host":"192.168.1.50"}]`-shaped JSON
/// text; the renderer parses it and never learns more than network names.
#[tauri::command]
pub async fn smartcast_discover() -> Result<String, String> {
    // The recv loop blocks, so it runs on the blocking pool: the async
    // command merely parks, and every other invoke keeps flowing.
    let discovered = tokio::task::spawn_blocking(discover_ssdp)
        .await
        .map_err(|_| "TV discovery stopped before finishing".to_owned())??;
    serde_json::to_string(&discovered)
        .map_err(|_| "Could not encode the discovered TV list".to_owned())
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
    let deadline = Instant::now() + DISCOVERY_WINDOW;
    let resend_at = Instant::now() + DISCOVERY_RESEND_AFTER;
    let mut resent = false;
    let mut buffer = [0u8; 4096];
    let mut discovered: Vec<DiscoveredTv> = Vec::new();
    // Non-blocking drain loop: WouldBlock and transient socket errors park
    // ~50ms, the deadline bounds every branch, and nothing here can panic.
    while discovered.len() < DISCOVERY_LIMIT && Instant::now() < deadline {
        if !resent && Instant::now() >= resend_at {
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
    Some(DiscoveredTv { name: format!("{name} ({host})"), host })
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

async fn execute(
    session: &Session,
    request: PlannedRequest,
    cancelled: &AtomicBool,
    cancel_notify: &Notify,
) -> Result<(u16, String), ()> {
    if cancelled.load(Ordering::Acquire)
        || request.timeout_millis < 250
        || request.timeout_millis > 60_000
        || request.max_response_bytes == 0
        || request.max_response_bytes > 8 * 1024 * 1024
        || request_origin(&request.url).ok().as_deref() != Some(session.origin.as_str())
    {
        return Err(());
    }
    let method = reqwest::Method::from_bytes(request.method.as_bytes()).map_err(|_| ())?;
    let mut builder = session
        .client
        .request(method, &request.url)
        .timeout(Duration::from_millis(request.timeout_millis));
    for (name, value) in request.headers {
        builder = builder.header(name, value);
    }
    if let Some(body) = request.body {
        builder = builder.json(&body);
    }
    let response = tokio::select! {
        response = builder.send() => response.map_err(|_| ())?,
        _ = wait_for_cancel(cancelled, cancel_notify) => return Err(()),
    };
    if response.status().is_redirection()
        || response
            .content_length()
            .is_some_and(|length| length > request.max_response_bytes)
    {
        return Err(());
    }
    let status = response.status().as_u16();
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    loop {
        if cancelled.load(Ordering::Acquire) {
            return Err(());
        }
        let chunk = tokio::select! {
            chunk = stream.next() => chunk,
            _ = wait_for_cancel(cancelled, cancel_notify) => return Err(()),
        };
        let Some(chunk) = chunk else { break };
        let chunk = chunk.map_err(|_| ())?;
        if bytes.len().saturating_add(chunk.len()) > request.max_response_bytes as usize {
            return Err(());
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes)
        .map(|body| (status, body))
        .map_err(|_| ())
}

async fn wait_for_cancel(cancelled: &AtomicBool, notify: &Notify) {
    while !cancelled.load(Ordering::Acquire) {
        notify.notified().await;
    }
}

fn configured_origin(input: &str) -> Result<String, String> {
    let url = parse_https(input)?;
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err("Invalid SmartCast TV origin".into());
    }
    let authority = input
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(input)
        .split('/')
        .next()
        .unwrap_or_default();
    let explicit_port = if authority.starts_with('[') {
        authority
            .find(']')
            .is_some_and(|end| authority.as_bytes().get(end + 1) == Some(&b':'))
    } else {
        authority.rsplit_once(':').is_some_and(|(_, port)| {
            !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit())
        })
    };
    origin_string(
        &url,
        explicit_port
            .then(|| url.port_or_known_default())
            .flatten()
            .unwrap_or(7345),
    )
}

fn request_origin(input: &str) -> Result<String, String> {
    let url = parse_https(input)?;
    if url.query().is_some() || url.fragment().is_some() {
        return Err("Invalid SmartCast TV origin".into());
    }
    origin_string(&url, url.port_or_known_default().unwrap_or(443))
}

fn parse_https(input: &str) -> Result<Url, String> {
    let url = if input.contains("://") {
        Url::parse(input)
    } else {
        Url::parse(&format!("https://{input}"))
    }
    .map_err(|_| "Invalid SmartCast TV origin")?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.host_str().is_none()
    {
        return Err("Invalid SmartCast TV origin".into());
    }
    Ok(url)
}

fn origin_string(url: &Url, port: u16) -> Result<String, String> {
    let host = url.host_str().ok_or("Invalid SmartCast TV origin")?;
    let host = if host.contains(':') {
        format!("[{host}]")
    } else {
        host.to_owned()
    };
    Ok(format!("https://{host}:{port}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_defaults_to_smartcast_port_and_preserves_explicit_port() {
        assert_eq!(
            configured_origin("192.0.2.10").unwrap(),
            "https://192.0.2.10:7345"
        );
        assert_eq!(
            configured_origin("https://example.test:443").unwrap(),
            "https://example.test:443"
        );
    }

    #[test]
    fn request_origin_ignores_path_but_rejects_credentials_and_queries() {
        assert_eq!(
            request_origin("https://192.0.2.10:7345/key_command/").unwrap(),
            "https://192.0.2.10:7345"
        );
        assert!(request_origin("https://token@192.0.2.10:7345/key").is_err());
        assert!(request_origin("https://192.0.2.10:7345/key?next=other").is_err());
    }

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
}
