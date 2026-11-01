"""Guard release archive names against stale or inconsistent product versions."""
import json
from pathlib import Path
import tempfile
import unittest

from package_md_viewer import source_version


class SourceVersionTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / 'reader').mkdir()

    def write_versions(self, tauri, cargo):
        (self.root / 'reader/tauri.conf.json').write_text(json.dumps({'version': tauri}))
        (self.root / 'reader/Cargo.toml').write_text(f'[package]\nversion = "{cargo}"\n')

    def test_current_version_is_used(self):
        self.write_versions('0.1.1', '0.1.1')
        self.assertEqual(source_version(self.root), '0.1.1')

    def test_disagreement_is_rejected(self):
        self.write_versions('0.1.1', '0.1.0')
        with self.assertRaisesRegex(ValueError, 'must match'):
            source_version(self.root)

    def test_invalid_filename_version_is_rejected(self):
        self.write_versions('../0.1.1', '0.1.1')
        with self.assertRaisesRegex(ValueError, 'major.minor.patch'):
            source_version(self.root)


if __name__ == '__main__':
    unittest.main()
