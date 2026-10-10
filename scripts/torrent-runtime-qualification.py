#!/usr/bin/env python3
"""Verify or unpack the pinned Windows test capsule; never launch the product."""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--extract', type=Path, help='Fresh directory for owned Windows test files')
args = parser.parse_args()
directory = root / 'vendor/torrent-runtime-qualification'
lock = json.loads((directory / 'lock.json').read_text())
assert lock['revision'] == (root / 'TORRENT_RUNTIME_REF').read_text().strip(), 'Qualification source pin mismatch'
assert lock['platform'] == 'windows-amd64' and lock['race'] and not lock['dirty']
for name, digest in lock['files'].items():
    assert name == 'windows-tests.zip'
    path = directory / name
    assert not path.is_symlink() and hashlib.sha256(path.read_bytes()).hexdigest() == digest, 'Capsule hash mismatch'
with zipfile.ZipFile(directory / 'windows-tests.zip') as bundle:
    entries = bundle.infolist()
    assert len(entries) == len(lock['contents']) and {entry.filename for entry in entries} == set(lock['contents'])
    for entry in entries:
        name = entry.filename
        assert not name.startswith('/') and '\\' not in name and ':' not in name
        assert all(part not in ('', '.', '..') for part in name.split('/'))
        assert hashlib.sha256(bundle.read(name)).hexdigest() == lock['contents'][name], 'Capsule content hash mismatch'
    if args.extract:
        args.extract.mkdir(parents=True, exist_ok=False)
        bundle.extractall(args.extract)
print('Windows qualification source and contents verified:', lock['revision'])
