#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

LOCK_PATH="${LOCK_PATH:-$ROOT_DIR/containers/versions/lock.json}"
SUMMARY_PATH="${SUMMARY_PATH:-$ROOT_DIR/artifacts/containers/hpc/frontend-smoke/summary.json}"

python3 - "$LOCK_PATH" "$SUMMARY_PATH" <<'PY'
import json
import os
import sys
from pathlib import Path

lock_path = Path(sys.argv[1])
summary_path = Path(sys.argv[2])

if not lock_path.exists():
    print(f"smoke lock gate: missing lock file {lock_path}", file=sys.stderr)
    raise SystemExit(1)
if not summary_path.exists():
    if "CI" in os.environ:
        print(f"smoke lock gate: missing smoke summary {summary_path}", file=sys.stderr)
        raise SystemExit(1)
    print(f"smoke lock gate: SKIP (missing smoke summary {summary_path})")
    raise SystemExit(0)

lock = json.loads(lock_path.read_text(encoding="utf-8"))
summary = json.loads(summary_path.read_text(encoding="utf-8"))
rows = {str(row.get("tool", "")).strip(): row for row in summary.get("items", [])}
errors = []

for item in lock.get("items", []):
    tool = str(item.get("tool", "")).strip()
    if not tool:
        continue
    row = rows.get(tool)
    if row is None:
        errors.append(f"{tool}: missing smoke summary row")
        continue
    if str(row.get("status", "")) != "ok":
        errors.append(f"{tool}: smoke status is not ok")
    log_dir = str(row.get("smoke_log_dir", "")).strip()
    if not log_dir:
        errors.append(f"{tool}: missing smoke_log_dir")
    else:
        path = Path(log_dir)
        if not path.exists():
            errors.append(f"{tool}: smoke_log_dir does not exist: {log_dir}")
        # Contracted path: artifacts/containers/smoke/<tool>/<timestamp>/
        normalized = path.as_posix()
        marker = f"/smoke/{tool}/"
        if marker not in normalized:
            errors.append(f"{tool}: smoke_log_dir not in required layout: {log_dir}")

if errors:
    print("smoke lock gate: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print(f"smoke lock gate: OK ({len(lock.get('items', []))} tools)")
PY
