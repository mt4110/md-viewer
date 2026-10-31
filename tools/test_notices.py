"""Regression checks for missing, stale and substituted distribution notices."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

import generate_notices as notices


class NoticeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def test_nested_and_british_spelling_are_collected(self):
        (self.root / "licenses").mkdir()
        (self.root / "licenses/LICENSE-MIT").write_text("MIT license body")
        (self.root / "LICENCE").write_text("BSD license body")
        (self.root / "license.rs").write_text("not a license document")
        self.assertEqual(set(notices.license_files(self.root)), {"LICENCE", "licenses/LICENSE-MIT"})

    def test_empty_license_is_an_error(self):
        (self.root / "LICENSE").write_text("  \n")
        with self.assertRaisesRegex(ValueError, "Empty license"):
            notices.license_files(self.root)

    def override(self, data=b"Pinned license\n"):
        directory = self.root / "tools/licenses"
        directory.mkdir(parents=True)
        (directory / "MIT.txt").write_bytes(data)
        return {"files": [{"file": "MIT.txt", "sha256": hashlib.sha256(data).hexdigest()}]}

    def test_modified_override_is_rejected(self):
        entry = self.override()
        (self.root / "tools/licenses/MIT.txt").write_text("Substituted text")
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            notices.pinned_texts(self.root, entry)

    def test_override_cannot_escape_license_directory(self):
        entry = self.override()
        entry["files"][0]["file"] = "../outside.txt"
        with self.assertRaisesRegex(ValueError, "escapes"):
            notices.pinned_texts(self.root, entry)

    def test_pinned_override_is_read(self):
        entry = self.override()
        self.assertEqual(notices.pinned_texts(self.root, entry), {"MIT.txt": "Pinned license\n"})

    def package(self):
        (self.root / "tools").mkdir()
        (self.root / "tools/license_overrides.json").write_text("{}")
        (self.root / "Cargo.lock").write_text('[[package]]\nname="example"\nversion="1.0.0"\nchecksum="abc"\n')
        manifest = self.root / "crate/Cargo.toml"
        manifest.parent.mkdir()
        manifest.write_text("")
        return {"name": "example", "version": "1.0.0", "license": "MIT", "repository": None,
                "source": "registry+https://github.com/rust-lang/crates.io-index", "manifest_path": str(manifest)}

    def test_missing_license_fails_before_packaging(self):
        package = self.package()
        with self.assertRaisesRegex(ValueError, "License text missing: example@1.0.0"):
            notices.collect_inventory(self.root, {"packages": [package]})
        self.assertFalse((self.root / notices.OUTPUT).exists())

    def test_override_for_another_revision_is_rejected(self):
        package = self.package()
        (self.root / "crate/.cargo_vcs_info.json").write_text('{"git":{"sha1":"new"}}')
        override = {"example@1.0.0": {"revision": "old", "license": "MIT", "files": []}}
        (self.root / "tools/license_overrides.json").write_text(json.dumps(override))
        with self.assertRaisesRegex(ValueError, "no longer matches"):
            notices.collect_inventory(self.root, {"packages": [package]})

    def test_source_access_information_is_in_readable_notices(self):
        item = {"name": "example", "version": "1.0", "license": "MPL-2.0",
                "source_url": "https://example.invalid/exact-version.crate", "source_sha256": "checksum",
                "license_texts": {"LICENSE": "Original license text"}}
        text = notices.render([item])["THIRD_PARTY_NOTICES.txt"]
        self.assertIn(item["source_url"], text)
        self.assertIn(item["source_sha256"], text)
        self.assertIn("Original license text", text)


if __name__ == "__main__":
    unittest.main()
