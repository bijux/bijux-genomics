#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1])
idx = root / "configs/hpc/rsync/index.md"
rsync_dir = root / "configs/hpc/rsync"
errors = []

if not idx.exists():
    errors.append("configs/hpc/rsync/index.md missing")
else:
    t = idx.read_text(encoding="utf-8")
    if "owner" not in t.lower():
        errors.append("configs/hpc/rsync/index.md must include owner for each pattern file")
    for p in sorted(rsync_dir.glob("*.txt")):
        if p.name not in t:
            errors.append(f"configs/hpc/rsync/index.md missing reference to {p.name}")

if errors:
    print("hpc-rsync-docs-parity: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("hpc-rsync-docs-parity: OK")
PY
