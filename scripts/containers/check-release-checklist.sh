#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
checklist = root / "containers/docs/RELEASE_CHECKLIST.md"
gate = root / "scripts/containers/release-gate.sh"
if not checklist.exists():
    print("release checklist check: missing containers/docs/RELEASE_CHECKLIST.md", file=sys.stderr)
    raise SystemExit(1)
if not gate.exists():
    print("release checklist check: missing scripts/containers/release-gate.sh", file=sys.stderr)
    raise SystemExit(1)

text = checklist.read_text(encoding="utf-8")
scripts = sorted(set(re.findall(r"`(scripts/containers/[^`]+\.sh)`", text)))
gate_text = gate.read_text(encoding="utf-8")
missing = [s for s in scripts if Path(root / s).exists() and s.split("/")[-1] not in gate_text]

if missing:
    print("release checklist check: release-gate missing checklist-mapped scripts:", file=sys.stderr)
    for s in missing:
        print(f"- {s}", file=sys.stderr)
    raise SystemExit(1)

print(f"release checklist mapping: OK ({len(scripts)} mapped scripts)")
PY
