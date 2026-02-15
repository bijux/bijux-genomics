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
import json
import sys
from pathlib import Path
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
profile_path = root / "configs/runtime/profiles/vcf_downstream_local.toml"
panels_path = root / "configs/vcf/panels/panels.toml"
panel_locks_path = root / "configs/vcf/panels/locks/panel_locks.toml"
panel_lock_json_path = root / "configs/vcf/panels/locks/lock.json"

errors = []
for p in (profile_path, panels_path, panel_locks_path, panel_lock_json_path):
    if not p.exists():
        errors.append(f"missing {p.relative_to(root)}")
if errors:
    print("enabled-vcf-panel-metadata: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

profile = tomllib.loads(profile_path.read_text(encoding="utf-8"))
enabled_panel = str(profile.get("reference_panel_id", "")).strip()
if not enabled_panel:
    errors.append("configs/runtime/profiles/vcf_downstream_local.toml: reference_panel_id must be non-empty")

panels = tomllib.loads(panels_path.read_text(encoding="utf-8")).get("panel", [])
panel_by_id = {str(p.get("id", "")).strip(): p for p in panels}
locks_toml = tomllib.loads(panel_locks_path.read_text(encoding="utf-8")).get("locks", {})
locks_json = json.loads(panel_lock_json_path.read_text(encoding="utf-8"))
lock_json_by_id = {
    str(p.get("id", "")).strip(): p
    for p in locks_json.get("panels", [])
    if str(p.get("id", "")).strip()
}

if enabled_panel and enabled_panel not in panel_by_id:
    errors.append(f"enabled panel {enabled_panel} missing from configs/vcf/panels/panels.toml")
if enabled_panel and enabled_panel not in locks_toml:
    errors.append(f"enabled panel {enabled_panel} missing lock entry in configs/vcf/panels/locks/panel_locks.toml")
if enabled_panel and enabled_panel not in lock_json_by_id:
    errors.append(f"enabled panel {enabled_panel} missing entry in configs/vcf/panels/locks/lock.json")

if enabled_panel and enabled_panel in panel_by_id:
    panel = panel_by_id[enabled_panel]
    license_value = str(panel.get("license", "")).strip()
    if not license_value:
        errors.append(f"enabled panel {enabled_panel} missing license in panels.toml")
    lock_ref = str(panel.get("lock_ref", "")).strip()
    if not lock_ref:
        errors.append(f"enabled panel {enabled_panel} missing lock_ref in panels.toml")

if enabled_panel and enabled_panel in lock_json_by_id and enabled_panel in panel_by_id:
    panel_license = str(panel_by_id[enabled_panel].get("license", "")).strip()
    lock_license = str(lock_json_by_id[enabled_panel].get("license", "")).strip()
    if not lock_license:
        errors.append(f"enabled panel {enabled_panel} missing license in lock.json")
    elif panel_license and panel_license != lock_license:
        errors.append(
            f"enabled panel {enabled_panel} license mismatch between panels.toml ({panel_license}) and lock.json ({lock_license})"
        )

if errors:
    print("enabled-vcf-panel-metadata: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("enabled-vcf-panel-metadata: OK")
PY
