#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

manifest="$ROOT_DIR/containers/TOOL_IDS.txt"
[[ -f "$manifest" ]] || { echo "missing $manifest" >&2; exit 1; }

python3 - "$manifest" <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
containers_root = path.parent
lines = path.read_text(encoding="utf-8").splitlines()
errors = []

required_headers = [
    "# GENERATED FILE - DO NOT EDIT",
    "# Regenerate with: scripts/containers/generate-tool-ids.sh",
    "# format: <tool_id><TAB><status>",
]
for i, header in enumerate(required_headers):
    if i >= len(lines) or lines[i] != header:
        errors.append(f"header line {i+1} mismatch: expected '{header}'")

seen = set()
status_by_id = {}
allowed_status = {"production", "experimental", "planned"}
for i, raw in enumerate(lines, start=1):
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    parts = raw.split("\t")
    if len(parts) != 2:
        errors.append(f"line {i}: expected exactly 2 TAB-separated fields")
        continue
    tool_id, status = parts[0].strip(), parts[1].strip()
    if not re.fullmatch(r"[a-z][a-z0-9_]*", tool_id):
        errors.append(f"line {i}: invalid tool_id '{tool_id}'")
    if status not in allowed_status:
        errors.append(f"line {i}: invalid status '{status}'")
    if tool_id in seen:
        errors.append(f"line {i}: duplicate tool_id '{tool_id}'")
    seen.add(tool_id)
    status_by_id[tool_id] = status

# Mapping contract:
# - production/experimental tools must map to exactly one Apptainer def and exactly one Dockerfile.
# - planned tools may be absent but cannot have duplicate mappings.
for tool_id, status in status_by_id.items():
    ap_defs = [
        containers_root / "apptainer" / "bijux" / f"{tool_id}.def",
        containers_root / "apptainer" / "non-bijux" / f"{tool_id}.def",
    ]
    docker_defs = [
        containers_root / "docker" / "arm64" / f"Dockerfile.{tool_id}",
        containers_root / "docker" / "amd64" / f"Dockerfile.{tool_id}",
    ]
    ap_count = sum(1 for p in ap_defs if p.exists())
    docker_count = sum(1 for p in docker_defs if p.exists())
    if status in {"production", "experimental"}:
        if ap_count != 1:
            errors.append(f"tool '{tool_id}' ({status}) must map to exactly one apptainer def (found {ap_count})")
        if docker_count != 1:
            errors.append(f"tool '{tool_id}' ({status}) must map to exactly one dockerfile (found {docker_count})")
    else:
        if ap_count > 1:
            errors.append(f"tool '{tool_id}' ({status}) has ambiguous apptainer defs (found {ap_count})")
        if docker_count > 1:
            errors.append(f"tool '{tool_id}' ({status}) has ambiguous dockerfiles (found {docker_count})")

if errors:
    print("tool id contract check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("tool id contract: OK")
PY
