#!/usr/bin/env python3
"""Build a source-only md-viewer archive in a NEW directory; preserve all originals."""
import argparse
import gzip
import hashlib
import io
import json
import re
import tomllib
from pathlib import Path
import tarfile
from generate_notices import generated_notices, OUTPUT

ROOT = Path(__file__).resolve().parents[1]


def source_version(root=ROOT):
    version = json.loads((root / 'reader/tauri.conf.json').read_text(encoding='utf-8'))['version']
    cargo_version = tomllib.loads((root / 'reader/Cargo.toml').read_text(encoding='utf-8'))['package']['version']
    if not isinstance(version, str) or re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+', version) is None:
        raise ValueError('Expected a numeric major.minor.patch release version')
    if version != cargo_version:
        raise ValueError('Cargo and Tauri versions must match')
    return version


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Choose a new output directory')
    version = source_version()
    notices = generated_notices()
    for name, text in notices.items():
        path = ROOT / OUTPUT / name
        if not path.is_file() or path.read_text(encoding='utf-8') != text:
            parser.error(f'Stale or missing notices: {path}; regenerate and review')
    names = ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'package.json',
             'package-lock.json', '.gitignore', '.gitattributes', '.markdownlint-cli2.jsonc',
             'SECURITY.md', 'LICENSE-SCOPE.md', 'tools/package_md_viewer.py', 'tools/build_dev_app.py',
             'tools/generate_notices.py', 'tools/license_overrides.json', 'tools/test_notices.py',
             'tools/test_packaging.py']
    names.extend(f.relative_to(ROOT).as_posix() for f in (ROOT / 'tools/licenses').rglob('*') if f.is_file())
    for directory in ['reader', '.cargo', '.github', 'docs/markdown-reader']:
        for file in (ROOT / directory).rglob('*'):
            relative = file.relative_to(ROOT)
            if file.is_file() and not any(part in ['target', 'gen', '__pycache__'] for part in relative.parts):
                names.append(relative.as_posix())
    entries = {}
    for name in sorted(set(names)):
        file = ROOT / name
        if file.is_symlink() or any(p.is_symlink() for p in file.parents if p != ROOT.parent):
            raise SystemExit('Symlink in source inputs')
        entries[name] = file.read_bytes()
    entries['README.md'] = (ROOT / 'docs/markdown-reader/DISTRIBUTION_README.md').read_bytes()
    for name, text in notices.items():
        entries[name] = text.encode()
    entries['SOURCE_MANIFEST.json'] = json.dumps({name: hashlib.sha256(data).hexdigest() for name, data in entries.items()}, indent=2).encode()
    args.output.mkdir(parents=True, exist_ok=False)
    archive = args.output / f'md-viewer-{version}-source.tar.gz'
    with archive.open('xb') as stream:
        with gzip.GzipFile(filename='', mode='wb', fileobj=stream, mtime=0) as zipped:
            with tarfile.open(fileobj=zipped, mode='w') as tar:
                for name, data in sorted(entries.items()):
                    item = tarfile.TarInfo('md-viewer/' + name)
                    item.size = len(data)
                    item.mode = 0o644
                    tar.addfile(item, io.BytesIO(data))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    (args.output / 'SHA256SUMS').write_text(f'{digest}  {archive.name}\n')
    print(json.dumps({'archive': str(archive), 'sha256': digest, 'license_text_missing': []}))


if __name__ == '__main__':
    main()
