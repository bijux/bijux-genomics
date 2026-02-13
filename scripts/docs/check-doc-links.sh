#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

python3 - <<'PY'
from pathlib import Path
import re
import sys

root = Path('.').resolve()
docs = root / 'docs'
pat = re.compile(r'\[[^\]]*\]\(([^)]+)\)')
errs = []

for md in docs.rglob('*.md'):
    text = md.read_text(encoding='utf-8')
    for target in pat.findall(text):
        target = target.strip()
        if not target or target.startswith(('http://', 'https://', 'mailto:')):
            continue
        if target.startswith('#'):
            continue
        target = target.split('#', 1)[0]
        if not target:
            continue
        if target.startswith('/'):
            cand = root / target.lstrip('/')
        else:
            cand = (md.parent / target).resolve()
        if not cand.exists():
            errs.append(f"{md.relative_to(root)} -> {target}")

if errs:
    print('docs link check failed:')
    for e in errs:
        print(f'  {e}')
    sys.exit(1)

print('docs links: OK')
PY
