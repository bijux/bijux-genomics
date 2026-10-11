#!/usr/bin/env python3
"""Verify source-owned MathJax components, fonts and immutable package provenance."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sys

MANIFEST_SHA256 = 'b55ec1239a37d8167adcaed601c36e0390dc0a150000a339ddb5e8a76b74a9eb'
ARCHIVE_SHA256 = '1b9c0a1c44df864e915690558e72adb9cc5203360daefd385084ced3b6c64c09'
ASSET_ROOT = 'docs/assets/vendor/mathjax'


def verify(root: Path) -> dict:
    root = root.resolve()
    manifest_path = root/'configs/docs/mathjax-source.json'
    if manifest_path.is_symlink() or not manifest_path.is_file():
        raise ValueError('MathJax requires a regular source-owned provenance manifest')
    content = manifest_path.read_bytes()
    if hashlib.sha256(content).hexdigest() != MANIFEST_SHA256:
        raise ValueError('MathJax package provenance differs from reviewed source')
    manifest = json.loads(content)
    if (manifest['schema'], manifest['package'], manifest['version'], manifest['asset_root'],
            manifest['archive']['sha256']) != (1, 'mathjax', '3.2.2', ASSET_ROOT, ARCHIVE_SHA256):
        raise ValueError('MathJax source package identity is unreviewed')
    assets = root/ASSET_ROOT
    if any(parent.is_symlink() for parent in (assets, *assets.parents) if parent != root):
        raise ValueError('MathJax source asset ancestors must not be symlinks')
    expected = set()
    for record in manifest['files']:
        name = record['path']
        relative = Path(name)
        if relative.is_absolute() or '..' in relative.parts or relative.as_posix() != name or name in expected:
            raise ValueError('MathJax source manifest has an unsafe or duplicate path')
        expected.add(name)
        path = assets/relative
        if path.is_symlink() or not path.is_file():
            raise ValueError(f'MathJax source asset is missing or irregular: {name}')
        data = path.read_bytes()
        if len(data) != record['bytes'] or hashlib.sha256(data).hexdigest() != record['sha256']:
            raise ValueError(f'MathJax source asset differs from its reviewed package: {name}')
    observed = set()
    for path in assets.rglob('*'):
        if path.is_symlink():
            raise ValueError('MathJax source asset tree must not contain symlinks')
        if path.is_file():
            observed.add(path.relative_to(assets).as_posix())
    if observed != expected:
        raise ValueError('MathJax source asset roster differs from its reviewed package')
    configuration = (root/'mkdocs.yml').read_text()
    scripts = configuration.split('extra_javascript:\n', 1)[1].split('\nwatch:', 1)[0]
    if '  - assets/vendor/mathjax/tex-mml-chtml.js\n' not in scripts or 'https://' in scripts:
        raise ValueError('MathJax publication must use the source-owned local component bundle')
    return {'package': manifest['package'], 'version': manifest['version'],
            'files': len(expected), 'manifest_sha256': MANIFEST_SHA256,
            'archive_sha256': ARCHIVE_SHA256}


if __name__ == '__main__':
    try:
        print(json.dumps(verify(Path(__file__).resolve().parents[2]), sort_keys=True))
    except (OSError, ValueError, KeyError, IndexError) as error:
        print(f'MathJax source verification failed: {error}', file=sys.stderr)
        raise SystemExit(1) from error
