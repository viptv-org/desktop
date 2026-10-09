import {readFileSync,lstatSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve,join} from 'node:path';
const root=resolve(import.meta.dirname,'..'),directory=resolve(root,'vendor/torrent-runtime');
const lock=JSON.parse(readFileSync(resolve(directory,'lock.json'),'utf8'));
if(!/^[a-f0-9]{40}$/.test(lock.revision)||readFileSync(resolve(root,'TORRENT_RUNTIME_REF'),'utf8').trim()!==lock.revision)throw Error('Runtime source pin mismatch');
for(const [name,expected] of Object.entries(lock.files)) {
  if(name.startsWith('/')||name.split('/').some(part=>!part||part==='.'||part==='..'))throw Error('Runtime artifact path invalid');
  const path=resolve(directory,name);
  if(lstatSync(path).isSymbolicLink()||createHash('sha256').update(readFileSync(path)).digest('hex')!==expected)throw Error('Runtime artifact integrity failed');
}
console.log('Desktop runtime source and artifact integrity passed');

const host = JSON.parse(readFileSync(join(root,'vendor/torrent-runtime-host/lock.json'),'utf8'));
if (host.revision !== readFileSync(join(root,'RUNTIME_HOST_REF'),'utf8').trim()) throw new Error('Runtime host pin mismatch');
for (const [name, expected] of Object.entries(host.files)) {
  if (name.startsWith('/') || name.split('/').includes('..')) throw new Error('Invalid host artifact path');
  if (lstatSync(join(root,'vendor/torrent-runtime-host',name)).isSymbolicLink() || createHash('sha256').update(readFileSync(join(root,'vendor/torrent-runtime-host',name))).digest('hex') !== expected) throw new Error('Runtime host artifact mismatch');
}
