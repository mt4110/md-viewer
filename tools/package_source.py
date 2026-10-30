#!/usr/bin/env python3
"""Create a local source handoff. Never upload, overwrite, or delete files."""
import gzip
import hashlib
import io
import json
from pathlib import Path
import sys
import tarfile

ROOT = Path(__file__).resolve().parent.parent

def main():
    if len(sys.argv) != 2:
        raise SystemExit('usage: python3 tools/package_source.py <new-output-directory>')
    output = Path(sys.argv[1])
    files = [ROOT / name for name in ['README.md', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.gitignore']]
    for folder in ['src', 'assets', 'fixtures', 'tests', 'docs', 'tools']:
        files.extend(p for p in (ROOT / folder).rglob('*') if p.is_file())
    entries = {}
    for path in sorted(files):
        if path.is_symlink() or any(p.is_symlink() for p in path.parents if p != ROOT.parent):
            raise SystemExit('symlinks are not supported')
        relative = path.relative_to(ROOT).as_posix()
        if '__pycache__' in path.parts:
            continue
        entries[relative] = path.read_bytes()
    manifest = {'schema_version': 1, 'files': {name: hashlib.sha256(data).hexdigest() for name, data in entries.items()}}
    entries['SOURCE_MANIFEST.json'] = (json.dumps(manifest, indent=2, ensure_ascii=False) + '\n').encode()
    output.mkdir(parents=True, exist_ok=False)
    archive = output / 'veil-boundary-lab-source.tar.gz'
    with archive.open('xb') as stream:
        with gzip.GzipFile(filename='', mode='wb', fileobj=stream, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode='w') as tar:
                for name, data in sorted(entries.items()):
                    info = tarfile.TarInfo('veil-boundary-lab/' + name)
                    info.size = len(data)
                    info.mode = 0o644
                    tar.addfile(info, io.BytesIO(data))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    with (output / 'SHA256SUMS').open('x') as stream:
        stream.write(f'{digest}  {archive.name}\n')
    print(f'{archive}\nsha256={digest}\nsource_files={len(manifest["files"])}')

if __name__ == '__main__':
    main()
