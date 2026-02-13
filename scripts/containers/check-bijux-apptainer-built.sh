#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import json
import sys

root = Path(sys.argv[1])
in_ci = bool(__import__("os").environ.get("CI"))
summary_path = root / "artifacts/containers/summary.json"
bijux_defs = sorted(p.stem for p in (root / "containers/apptainer/bijux").glob("*.def"))

if not in_ci:
    print("bijux apptainer built: SKIP (CI-only gate)")
    raise SystemExit(0)

if not summary_path.exists():
    print("bijux apptainer built: missing artifacts/containers/summary.json", file=sys.stderr)
    raise SystemExit(1)

summary = json.loads(summary_path.read_text(encoding="utf-8"))
rows = {}
for item in summary.get("items", []):
    tool = str(item.get("tool", "")).strip()
    runtime = str(item.get("runtime", "")).strip()
    if runtime != "apptainer" or not tool:
        continue
    rows[tool] = item

errors = []
for tool in bijux_defs:
    row = rows.get(tool)
    if not row:
        errors.append(f"{tool}: missing apptainer summary row")
        continue
    if str(row.get("status", "")).strip() != "ok":
        errors.append(f"{tool}: apptainer status is not ok")
        continue
    manifest_path = Path(str(row.get("manifest", "")))
    if not manifest_path.exists():
        errors.append(f"{tool}: missing manifest at {manifest_path}")
        continue
    m = json.loads(manifest_path.read_text(encoding="utf-8"))
    sif_sha = str(m.get("resolved_image_digest", "")).strip()
    if not sif_sha:
        errors.append(f"{tool}: missing resolved_image_digest (sif sha256) in manifest")

if errors:
    print("bijux apptainer built: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("bijux apptainer built: OK")
PY
