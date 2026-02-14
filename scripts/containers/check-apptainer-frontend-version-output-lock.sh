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
import hashlib
import json
import os
import sys

root = Path(sys.argv[1])
summary = root / "artifacts/containers/hpc/frontend-smoke/summary.json"
lock = root / "containers/versions/lock.json"

if not lock.exists():
    print("frontend version-output lock check: missing lock.json", file=sys.stderr)
    raise SystemExit(1)
if not summary.exists():
    if "CI" in os.environ:
        print("frontend version-output lock check: missing frontend smoke summary in CI", file=sys.stderr)
        raise SystemExit(1)
    print("frontend version-output lock check: SKIP (no frontend smoke summary)")
    raise SystemExit(0)

s = json.loads(summary.read_text(encoding="utf-8"))
l = json.loads(lock.read_text(encoding="utf-8"))
lock_map = {str(i.get("tool", "")).strip(): str(i.get("frontend_smoke_version_output_sha256", "")).strip() for i in l.get("items", [])}
errors = []
for row in s.get("items", []):
    tool = str(row.get("tool", "")).strip()
    out = str(row.get("normalized_version_output", "") or row.get("version_output", "")).strip().lower()
    if not tool:
        continue
    if str(row.get("status", "")) != "ok":
        errors.append(f"{tool}: smoke status is not ok")
        continue
    if not out:
        errors.append(f"{tool}: empty version output in frontend smoke summary")
        continue
    current = hashlib.sha256(out.encode("utf-8")).hexdigest()
    locked = lock_map.get(tool, "")
    if not locked:
        errors.append(f"{tool}: missing frontend_smoke_version_output_sha256 in lock")
    elif current != locked:
        errors.append(f"{tool}: frontend version output drift detected; regenerate lock")

if errors:
    print("frontend version-output lock check: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("frontend version-output lock check: OK")
PY
