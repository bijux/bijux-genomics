#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
pat = re.compile(r'\[[^\]]*\]\(([^)]+)\)')
errors = []

for md in sorted((root / "domain").glob("*/docs/*.md")):
    text = md.read_text(encoding="utf-8")
    for target in pat.findall(text):
        t = target.strip()
        if not t or t.startswith(("http://", "https://", "mailto:", "#")):
            continue
        t = t.split("#", 1)[0]
        cand = (md.parent / t).resolve()
        if not cand.exists():
            errors.append(f"{md.relative_to(root)} -> {target}")

if errors:
    print("domain docs link check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("domain docs links: OK")
PY
