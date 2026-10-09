import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const mode = process.argv[2];
if (!['dev', 'build'].includes(mode)) throw Error('Use dev or build');
const runtime = spawnSync(process.execPath, [fileURLToPath(new URL('./torrent-runtime-check.mjs', import.meta.url))], { stdio: 'inherit' });
if (runtime.status !== 0) process.exit(runtime.status ?? 1);
const args = ['run' , '--prefix', 'tv', mode];
// Hosted TV-web builds use /tv/. The native package serves frontendDist at /.
if (mode === 'build') args.push('--', '--base', '/');
const result = spawnSync(process.platform === 'win32' ? 'npm.cmd' : 'npm', args, {
  stdio: 'inherit', shell: process.platform === 'win32',
  env: { ...process.env, VITE_VIZIO_RECEIVER_URL: 'https://watch.syek.tech/?platform=vizio' },
});
if (result.error) throw result.error;
if (mode === 'build' && result.status === 0) {
  const check = spawnSync(process.execPath, [fileURLToPath(new URL('./check-frontend.mjs', import.meta.url))], { stdio: 'inherit' });
  if (check.error) throw check.error;
  if (check.status !== 0) process.exit(check.status ?? 1);
}
process.exit(result.status ?? 1);
