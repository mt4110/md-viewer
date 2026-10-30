#!/usr/bin/env python3
"""Create a fresh, ad-hoc-signed macOS development bundle. Never replace a bundle."""
import argparse
import pathlib
import plistlib
import shutil
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=pathlib.Path, required=True)
    args = parser.parse_args()
    if sys.platform != 'darwin' or args.out.suffix != '.app':
        parser.error('macOS and a new .app output path are required')
    bundle = args.out.absolute()
    if bundle.exists() or bundle.is_symlink():
        parser.error('Output exists; choose a new .app path')
    root = pathlib.Path(__file__).resolve().parents[1]
    subprocess.run(['rustup', 'run', '1.95.0', 'cargo', 'build', '--manifest-path',
                    str(root / 'reader/Cargo.toml'), '--locked'], check=True)
    bundle.mkdir(parents=True, exist_ok=False)
    executable = bundle / 'Contents/MacOS'
    executable.mkdir(parents=True)
    shutil.copy2(root / 'reader/target/debug/md-viewer', executable / 'md-viewer')
    with (bundle / 'Contents/Info.plist').open('xb') as file:
        plistlib.dump({'CFBundleExecutable': 'md-viewer',
                       'CFBundleIdentifier': 'io.github.mt4110.mdviewer',
                       'CFBundleName': 'md-viewer', 'CFBundlePackageType': 'APPL',
                       'CFBundleVersion': '0.1.0', 'CFBundleShortVersionString': '0.1.0',
                       'NSHighResolutionCapable': True,
                       'CFBundleDocumentTypes': [{'CFBundleTypeName': 'Markdown',
                         'CFBundleTypeRole': 'Viewer', 'CFBundleTypeExtensions': ['md', 'markdown']}]}, file)
    subprocess.run(['/usr/bin/codesign', '--force', '--sign', '-', '--timestamp=none', str(bundle)], check=True)
    subprocess.run(['/usr/bin/codesign', '--verify', '--strict', '--verbose=2', str(bundle)], check=True)
    print(bundle)
    print('Local development signature only; not Developer ID signing or notarization.')


if __name__ == '__main__':
    main()
