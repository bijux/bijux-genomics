"""Exercise provenance and source-asset refusal without executing scientific pipelines."""
from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location('bijux_mathjax_assets', ROOT/'configs/docs/verify_mathjax_assets.py')
assets = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(assets)


class MathJaxAssets(unittest.TestCase):
    def setUp(self):
        artifacts = ROOT/'artifacts/docs-publication/mathjax-controls'
        artifacts.mkdir(parents=True, exist_ok=True)
        self.scratch = tempfile.TemporaryDirectory(dir=artifacts)
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        shutil.copytree(ROOT/assets.ASSET_ROOT, self.root/assets.ASSET_ROOT)
        (self.root/'configs/docs').mkdir(parents=True)
        shutil.copyfile(ROOT/'configs/docs/mathjax-source.json', self.root/'configs/docs/mathjax-source.json')
        shutil.copyfile(ROOT/'mkdocs.yml', self.root/'mkdocs.yml')

    def rejected(self):
        with self.assertRaises(ValueError):
            assets.verify(self.root)

    def test_reviewed_source_and_native_font_component_roster_pass(self):
        self.assertEqual(assets.verify(self.root)['files'], 94)

    def test_public_license_preserves_immutable_package_bytes(self):
        source = json.loads((self.root/'configs/docs/mathjax-source.json').read_text())
        license = next(record for record in source['files'] if record['path'] == 'LICENSE.txt')
        self.assertEqual(license['archive_member'], 'package/LICENSE')
        self.assertEqual(hashlib.sha256((self.root/assets.ASSET_ROOT/'LICENSE.txt').read_bytes()).hexdigest(),
                         'cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30')

    def test_missing_native_font_is_refused(self):
        next((self.root/assets.ASSET_ROOT).rglob('*.woff')).unlink()
        self.rejected()

    def test_changed_native_entry_bundle_is_refused(self):
        (self.root/assets.ASSET_ROOT/'tex-mml-chtml.js').write_text('alert(1);')
        self.rejected()

    def test_changed_package_provenance_is_refused(self):
        path = self.root/'configs/docs/mathjax-source.json'
        path.write_bytes(path.read_bytes()+b'\n')
        self.rejected()

    def test_extra_executable_component_is_refused(self):
        (self.root/assets.ASSET_ROOT/'foreign.js').write_text('alert(1);')
        self.rejected()

    def test_source_asset_symlink_is_refused(self):
        path = self.root/assets.ASSET_ROOT/'tex-mml-chtml.js'
        path.unlink()
        path.symlink_to(ROOT/assets.ASSET_ROOT/'tex-mml-chtml.js')
        self.rejected()

    def test_floating_remote_math_bundle_is_refused(self):
        path = self.root/'mkdocs.yml'
        path.write_text(path.read_text().replace('assets/vendor/mathjax/tex-mml-chtml.js', 'https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js'))
        self.rejected()


if __name__ == '__main__':
    unittest.main()
