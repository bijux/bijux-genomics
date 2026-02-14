#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

out="${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/vuln_scan_report.json"
"$SCRIPT_DIR/check-vuln-allowlist.sh"
TOOLKIT="${TOOLKIT:-fastq_core}" PROMOTED_ONLY="${PROMOTED_ONLY:-1}" \
  "$SCRIPT_DIR/vuln-scan-hook.sh" "${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/sbom" "$out" >/dev/null
if [[ ! -f "$out" ]]; then
  echo "vuln hook: missing report $out" >&2
  exit 1
fi
python3 - "$ROOT_DIR" "$out" <<'PY'
from pathlib import Path
import json
import os
import sys

root = Path(sys.argv[1])
out = Path(sys.argv[2])
payload = json.loads(out.read_text(encoding="utf-8"))
items = payload.get("items", [])
rows = {str(i.get("tool", "")).strip(): i for i in items if str(i.get("tool", "")).strip()}
errors = []

lock = root / "containers/versions/lock.json"
if lock.exists():
    lock_data = json.loads(lock.read_text(encoding="utf-8"))
    promoted = sorted(
        str(i.get("tool", "")).strip()
        for i in lock_data.get("items", [])
        if str(i.get("status", "")).strip() == "production"
    )
else:
    promoted = []

if not rows and "CI" not in os.environ:
    print("vuln hook: SKIP (no local vuln scan items)")
    raise SystemExit(0)

if (os.environ.get("PROMOTED_ONLY", "1").strip() in {"1", "true", "yes"}) and promoted:
    for tool in promoted:
        row = rows.get(tool)
        if not row:
            errors.append(f"{tool}: missing vuln scan item for promoted tool")
            continue
        if str(row.get("status", "")).strip() not in {"ok", "not_scanned"}:
            errors.append(f"{tool}: vuln scan status is {row.get('status')}")
        per_tool = root / "artifacts/containers/vuln" / f"{tool}.json"
        if not per_tool.exists():
            errors.append(f"{tool}: missing per-tool vuln summary {per_tool}")

if errors:
    print("vuln hook: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
PY
echo "vuln hook: OK"
