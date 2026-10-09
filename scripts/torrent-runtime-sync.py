#!/usr/bin/env python3
"""Pin supervised Linux/Windows workers from one immutable shared runtime source."""
import hashlib
import json
from pathlib import Path
import shutil
import sys
ROOT=Path(__file__).resolve().parents[1]
DEST=ROOT/'vendor/torrent-runtime'
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
if sys.argv[1:2]==['sync']:
    products=Path(sys.argv[2]).resolve()
    reference=Path(sys.argv[3]).resolve()
    manifests=[json.loads((products/platform/'build.json').read_text()) for platform in ['host','windows']]
    assert not any(manifest['dirty'] for manifest in manifests)
    ref=json.loads((reference/'lock.json').read_text())
    assert manifests[0]['revision']==manifests[1]['revision']==ref['revision']
    DEST.mkdir(parents=True,exist_ok=True)
    for index,(platform,name) in enumerate([('host','torrent-worker'),('windows','torrent-worker.exe')]):
        source=products/platform/name
        assert digest(source)==manifests[index]['files'][name]
        shutil.copy2(source,DEST/name)
    for name in ['LICENSE','PROVENANCE.md','go.mod','go.sum']:
        shutil.copyfile(reference/name,DEST/name)
    shutil.copytree(reference/'licenses',DEST/'licenses',dirs_exist_ok=True)
    files={str(path.relative_to(DEST)):digest(path) for path in sorted(DEST.rglob('*')) if path.is_file() and path.name!='lock.json'}
    (DEST/'lock.json').write_text(json.dumps({'revision':ref['revision'],'repository':ref['repository'],'files':files},indent=2)+'\n')
    (ROOT/'TORRENT_RUNTIME_REF').write_text(ref['revision']+'\n')
lock=json.loads((DEST/'lock.json').read_text())
assert (ROOT/'TORRENT_RUNTIME_REF').read_text().strip()==lock['revision']
for name,value in lock['files'].items():
    assert not Path(name).is_absolute() and '..' not in Path(name).parts
    assert not (DEST/name).is_symlink() and digest(DEST/name)==value
print('Desktop shared runtime artifact integrity passed')
