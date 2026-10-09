import {readFileSync,lstatSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'..'),directory=resolve(root,'vendor/torrent-runtime');
const lock=JSON.parse(readFileSync(resolve(directory,'lock.json'),'utf8'));
if(!/^[a-f0-9]{40}$/.test(lock.revision)||readFileSync(resolve(root,'TORRENT_RUNTIME_REF'),'utf8').trim()!==lock.revision)throw Error('Runtime source pin mismatch');
for(const [name,expected] of Object.entries(lock.files)) {
  if(name.startsWith('/')||name.split('/').some(part=>!part||part==='.'||part==='..'))throw Error('Runtime artifact path invalid');
  const path=resolve(directory,name);
  if(lstatSync(path).isSymbolicLink()||createHash('sha256').update(readFileSync(path)).digest('hex')!==expected)throw Error('Runtime artifact integrity failed');
}
console.log('Desktop runtime source and artifact integrity passed');
