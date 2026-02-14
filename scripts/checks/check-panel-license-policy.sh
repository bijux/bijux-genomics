#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" || "${1:-}" == "--dry-run" || "${1:-}" == "--verbose" ]]; then
  cat <<'USAGE'
Usage: scripts/checks/check-panel-license-policy.sh
USAGE
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import json
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
panels_path = root / "configs/vcf/panels/panels.toml"
maps_path = root / "configs/vcf/maps/maps.toml"
lock_path = root / "configs/vcf/panels/locks/lock.json"
licenses_dir = root / "containers/licenses"

allowed = {
    "CC0-1.0",
    "CC-BY-4.0",
    "ODC-By-1.0",
    "MIT",
    "BSD-3-Clause",
}
forbidden = {
    "Unknown",
    "Proprietary",
    "All Rights Reserved",
}

errors = []
if not panels_path.exists():
    errors.append(f"missing {panels_path.relative_to(root)}")
if not maps_path.exists():
    errors.append(f"missing {maps_path.relative_to(root)}")
if not lock_path.exists():
    errors.append(f"missing {lock_path.relative_to(root)}")
if errors:
    print("panel-license-policy: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

panels = tomllib.loads(panels_path.read_text(encoding="utf-8")).get("panel", [])
maps = tomllib.loads(maps_path.read_text(encoding="utf-8")).get("map", [])
lock = json.loads(lock_path.read_text(encoding="utf-8"))
by_id = {str(item.get("id", "")).strip(): item for item in lock.get("panels", []) if isinstance(item, dict)}
tool_ids = set()

for panel in panels:
    pid = str(panel.get("id", "")).strip()
    if not pid:
        continue
    lic = str(panel.get("license", "")).strip()
    if not lic:
        errors.append(f"panel {pid}: missing license in panels.toml")
    elif lic in forbidden:
        errors.append(f"panel {pid}: forbidden license '{lic}'")
    elif lic not in allowed:
        errors.append(f"panel {pid}: unknown/unapproved license '{lic}'")

    citation = str(panel.get("citation", "")).strip()
    if not citation:
        errors.append(f"panel {pid}: missing citation in panels.toml")

    lock_entry = by_id.get(pid, {})
    if not lock_entry:
        errors.append(f"panel {pid}: missing entry in lock.json")
        continue
    lock_license = str(lock_entry.get("license", "")).strip()
    if lock_license != lic:
        errors.append(f"panel {pid}: license mismatch between panels.toml and lock.json")
    compat = panel.get("compatibility", {})
    tags = compat.get("tool_tags", []) if isinstance(compat, dict) else []
    for tag in tags:
        tool_ids.add(str(tag).strip())

for m in maps:
    mid = str(m.get("id", "")).strip()
    compat = m.get("compatibility", {})
    tags = compat.get("tool_tags", []) if isinstance(compat, dict) else []
    if not tags:
        errors.append(f"map {mid}: compatibility.tool_tags missing")
    for tag in tags:
        tool_ids.add(str(tag).strip())

for tool_id in sorted(t for t in tool_ids if t):
    lic_file = licenses_dir / f"{tool_id}.license.toml"
    if not lic_file.exists():
        errors.append(f"tool {tool_id}: missing containers/licenses/{tool_id}.license.toml")
        continue
    lic = tomllib.loads(lic_file.read_text(encoding="utf-8"))
    spdx = str(lic.get("spdx", "")).strip()
    upstream_license_id = str(lic.get("upstream_license_id", "")).strip()
    if not spdx:
        errors.append(f"tool {tool_id}: missing spdx in license metadata")
    if not upstream_license_id:
        errors.append(f"tool {tool_id}: missing upstream_license_id in license metadata")
    if spdx in forbidden or upstream_license_id in forbidden:
        errors.append(f"tool {tool_id}: forbidden license in metadata")

if errors:
    print("panel-license-policy: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("panel-license-policy: OK")
PY
