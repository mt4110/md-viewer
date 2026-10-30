#!/usr/bin/env python3
"""Build a source-only md-viewer archive in a NEW directory; preserve all originals."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Choose a new output directory')
    names = ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'package.json',
             'package-lock.json', '.gitignore', '.markdownlint-cli2.jsonc',
             'SECURITY.md', 'LICENSE-SCOPE.md', 'tools/package_md_viewer.py', 'tools/build_dev_app.py']
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
    metadata = json.loads(subprocess.check_output(['rustup', 'run', '1.95.0', 'cargo', 'metadata', '--locked', '--format-version', '1'], cwd=ROOT))
    inventory = []
    for package in metadata['packages']:
        if package['name'] == 'md-viewer':
            continue
        base = Path(package['manifest_path']).parent
        texts = {}
        for f in base.iterdir():
            if f.is_file() and f.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE')) and f.stat().st_size < 2*1024*1024:
                texts[f.name] = f.read_text(errors='replace')
        inventory.append({'name': package['name'], 'version': package['version'], 'license': package['license'],
                          'repository': package['repository'], 'license_texts': texts})
    entries['THIRD_PARTY_NOTICES.json'] = json.dumps(inventory, ensure_ascii=False, indent=2).encode()
    entries['SOURCE_MANIFEST.json'] = json.dumps({name: hashlib.sha256(data).hexdigest() for name, data in entries.items()}, indent=2).encode()
    args.output.mkdir(parents=True, exist_ok=False)
    archive = args.output / 'md-viewer-0.1.0-source.tar.gz'
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
    missing = [p['name'] for p in inventory if not p['license_texts']]
    print(json.dumps({'archive': str(archive), 'sha256': digest, 'license_text_missing': missing}))


if __name__ == '__main__':
    main()
