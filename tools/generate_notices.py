#!/usr/bin/env python3
"""Generate/check notices for the locked Windows x64 distribution, without downloads."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
TARGET = "x86_64-pc-windows-msvc"
OUTPUT = Path("reader/resources/notices")
MAX_TEXT = 2 * 1024 * 1024


def read_text(path):
    if path.is_symlink() or path.stat().st_size > MAX_TEXT:
        raise ValueError(f"Invalid license file: {path}")
    text = path.read_text(encoding="utf-8-sig")
    if not text.strip():
        raise ValueError(f"Empty license file: {path}")
    return text.replace("\r\n", "\n")


def license_files(base):
    prefixes = ("LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT", "AUTHORS", "UNLICENSE")
    result = {}
    for file in sorted(base.rglob("*"), key=lambda p: p.relative_to(base).as_posix()):
        if not file.is_file() or not file.name.upper().startswith(prefixes):
            continue
        if file.suffix.lower() in (".rs", ".c", ".h", ".py", ".toml", ".json"):
            continue
        if base.is_symlink() or any((base / parent).is_symlink() for parent in file.relative_to(base).parents):
            raise ValueError(f"Symlink in license path: {file}")
        result[file.relative_to(base).as_posix()] = read_text(file)
    return result


def pinned_texts(root, entry):
    texts = {}
    for item in entry["files"]:
        relative = Path(item["file"])
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError("License override path escapes tools/licenses")
        path = root / "tools/licenses" / relative
        directory = root / "tools/licenses"
        if directory.is_symlink() or path.is_symlink() or any((directory / parent).is_symlink() for parent in relative.parents):
            raise ValueError(f"Symlink in license override: {relative}")
        if hashlib.sha256(path.read_bytes()).hexdigest() != item["sha256"]:
            raise ValueError(f"License override hash mismatch: {relative}")
        texts[relative.as_posix()] = read_text(path)
    return texts


def collect_inventory(root, metadata):
    overrides = json.loads((root / "tools/license_overrides.json").read_text())
    locked = tomllib.loads((root / "Cargo.lock").read_text())
    checksums = {(p["name"], p["version"]): p.get("checksum") for p in locked["package"]}
    inventory = []
    for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
        if package["name"] == "md-viewer":
            continue
        key = f"{package['name']}@{package['version']}"
        if package["source"] != "registry+https://github.com/rust-lang/crates.io-index":
            raise ValueError(f"Review source and notices for {key}")
        checksum = checksums.get((package["name"], package["version"]))
        if not checksum or not package["license"]:
            raise ValueError(f"Missing locked checksum or license: {key}")
        base = Path(package["manifest_path"]).parent
        texts = license_files(base)
        override = overrides.get(key)
        if override:
            vcs = json.loads((base / ".cargo_vcs_info.json").read_text())
            if override["revision"] != vcs["git"]["sha1"] or override["license"] != package["license"]:
                raise ValueError(f"License override no longer matches {key}")
            texts.update(pinned_texts(root, override))
        if not texts:
            raise ValueError(f"License text missing: {key}; review a pinned upstream source")
        inventory.append({
            "name": package["name"], "version": package["version"],
            "license": package["license"], "repository": package["repository"],
            "source_url": f"https://static.crates.io/crates/{package['name']}/{package['name']}-{package['version']}.crate",
            "source_sha256": checksum,
            "license_sources": override["files"] if override else [],
            "license_texts": texts,
        })
    # Non-Rust material copied into the statically linked WebView2 loader.
    extra = overrides["Microsoft.Web.WebView2@1.0.3800.47"]
    sys = next(p for p in metadata["packages"] if p["name"] == "webview2-com-sys")
    loader = Path(sys["manifest_path"]).parent / "x64/WebView2LoaderStatic.lib"
    if hashlib.sha256(loader.read_bytes()).hexdigest() != extra["loader_sha256"]:
        raise ValueError("Review the Microsoft WebView2 SDK version and license")
    inventory.append({"name": "Microsoft.Web.WebView2", "version": "1.0.3800.47",
                      "license": "BSD-3-Clause (Microsoft SDK license text)", "repository": None,
                      "source_url": extra["source_url"], "source_sha256": extra["source_sha256"],
                      "license_sources": extra["files"], "license_texts": pinned_texts(root, extra)})
    helper = overrides["nsis-tauri-utils@0.5.3"]
    inventory.append({"name": "nsis-tauri-utils", "version": "0.5.3", "license": helper["license"],
                      "repository": "https://github.com/tauri-apps/nsis-tauri-utils",
                      "source_url": helper["source_url"], "source_sha256": None,
                      "source_revision": helper["revision"], "license_sources": helper["files"],
                      "license_texts": pinned_texts(root, helper)})
    return sorted(inventory, key=lambda p: (p["name"], p["version"]))


def render(inventory):
    header = ("md-viewer — Windows x64 third-party notices\n"
              "Scope: locked Windows dependencies, including build-time dependencies.\n"
              "This inventory is not a claim that every package is linked into the executable.\n"
              "The md-viewer MIT license is provided separately as LICENSE.txt.\n"
              "Exact crate source archives, the Microsoft SDK package and the NSIS helper source are listed below;\n"
              "these links also provide the corresponding MPL-covered source.\n"
              "Registry crates are unmodified. Third-party terms remain in effect.\n")
    sections = [header]
    for p in inventory:
        lines = ["=" * 72, f"{p['name']} {p['version']}", f"License: {p['license']}", f"Source: {p['source_url']}"]
        if p["source_sha256"]:
            lines.append(f"Source SHA-256: {p['source_sha256']}")
        if p.get("source_revision"):
            lines.append(f"Source revision: {p['source_revision']}")
        for file, text in sorted(p["license_texts"].items()):
            lines.extend([f"\n--- {file} ---", text.rstrip()])
        sections.append("\n".join(lines))
    return {"THIRD_PARTY_NOTICES.json": json.dumps(inventory, ensure_ascii=False, indent=2) + "\n",
            "THIRD_PARTY_NOTICES.txt": "\n\n".join(sections) + "\n"}


def generated_notices(root=ROOT, offline=False):
    command = ["rustup", "run", "1.95.0", "cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", TARGET]
    if offline:
        command.append("--offline")
    metadata = json.loads(subprocess.check_output(command, cwd=root))
    return render(collect_inventory(root, metadata))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    try:
        files = generated_notices(offline=args.offline)
        for name, text in files.items():
            file = ROOT / OUTPUT / name
            if args.check:
                if not file.is_file() or file.read_text(encoding="utf-8") != text:
                    raise ValueError(f"Stale notices: {file}; regenerate and review")
            else:
                file.parent.mkdir(parents=True, exist_ok=True)
                file.write_text(text, encoding="utf-8", newline="\n")
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Notice verification failed: {error}\n")
    print(f"Windows notices {'verified' if args.check else 'generated'}")


if __name__ == "__main__":
    main()
