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
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
owners_path = root / "containers/OWNERS.toml"
tool_ids_path = root / "containers/TOOL_IDS.txt"

if not owners_path.exists():
    print("missing containers/OWNERS.toml", file=sys.stderr)
    raise SystemExit(1)

owners_data = tomllib.loads(owners_path.read_text(encoding="utf-8"))
owner_rows = owners_data.get("owner", [])
if not owner_rows:
    print("containers/OWNERS.toml has no [[owner]] rows", file=sys.stderr)
    raise SystemExit(1)

rows = []
for row in owner_rows:
    tid = str(row.get("tool_id", "")).strip()
    team = str(row.get("team", "")).strip()
    contact = str(row.get("contact", "")).strip()
    if not tid or not team or not contact:
        print("each [[owner]] row must include tool_id, team, contact", file=sys.stderr)
        raise SystemExit(1)
    rows.append((tid, team))

tool_ids = []
for raw in tool_ids_path.read_text(encoding="utf-8").splitlines():
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    tool_ids.append(raw.split("\t", 1)[0].strip())

errors = []
for tool_id in tool_ids:
    matches = 0
    for pat, _team in rows:
        if pat == "*" or pat == tool_id:
            matches += 1
    if matches != 1:
        errors.append(f"{tool_id}: expected exactly one owner match, got {matches}")

if errors:
    print("container owners check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("container owners: OK")
PY
