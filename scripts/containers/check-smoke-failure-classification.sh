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
import json
import sys

root = Path(sys.argv[1])
manifests = root / "artifacts/containers/manifests"
if not manifests.exists():
    print("smoke failure classification: SKIP (no manifests)")
    raise SystemExit(0)

allowed = {"build", "runtime", "smoke_mismatch"}
errors = []
for path in sorted(manifests.glob("*.json")):
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        errors.append(f"{path.relative_to(root)}: invalid JSON")
        continue
    status = str(data.get("status", "")).strip()
    if status == "fail":
        fc = str(data.get("fail_class", "")).strip()
        if fc not in allowed:
            errors.append(f"{path.relative_to(root)}: missing/invalid fail_class '{fc}'")

if errors:
    print("smoke failure classification: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("smoke failure classification: OK")
PY
