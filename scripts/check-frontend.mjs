import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// Tauri serves frontendDist at its origin root. A hosted /tv/ prefix would
// request missing assets and leave the packaged webview blank.
const dist = resolve(process.argv[2] ?? fileURLToPath(new URL('../tv/dist', import.meta.url)));
const html = readFileSync(resolve(dist, 'index.html'), 'utf8');
const references = [...html.matchAll(/<(?:script|link)\b[^>]*\b(?:src|href)="([^"]+)"/g)];
if (!references.length) throw Error('The desktop entry contains no frontend assets');
for (const [, reference] of references) {
  const url = new URL(reference, 'https://tauri.localhost/');
  if (url.origin !== 'https://tauri.localhost') throw Error(`External desktop entry asset: ${reference}`);
  const path = resolve(dist, `.${decodeURIComponent(url.pathname)}`);
  if (!existsSync(path)) throw Error(`Desktop entry asset is missing from frontendDist: ${reference}`);
}
console.log(`Desktop entry assets resolve at the native origin root (${references.length} checked)`);
