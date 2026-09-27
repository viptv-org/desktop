import { spawnSync } from 'node:child_process';
const mode = process.argv[2];
if (!['dev', 'build'].includes(mode)) throw Error('Use dev or build');
const result = spawnSync(process.platform === 'win32' ? 'npm.cmd' : 'npm', ['run', '--prefix', 'tv', mode], {
  stdio: 'inherit', shell: process.platform === 'win32',
  env: { ...process.env, VITE_VIPTV_LOCAL_MODE: '1', VITE_VIZIO_RECEIVER_URL: 'https://watch.syek.tech/?platform=vizio' },
});
if (result.error) throw result.error;
process.exit(result.status ?? 1);
