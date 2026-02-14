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
import hashlib
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
maps_path = root / "configs/vcf/maps/maps.toml"
map_locks_path = root / "configs/vcf/maps/locks/map_locks.toml"
example_paths = [
    root / "examples/vcf/downstream-vcf-full-mini/example.toml",
]

errors = []
for p in (profile_path, panels_path, panel_locks_path, maps_path, map_locks_path):
    if not p.exists():
        errors.append(f"missing {p.relative_to(root)}")
if errors:
    print("vcf-reference-governance: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

profile = tomllib.loads(profile_path.read_text(encoding="utf-8"))
panels = tomllib.loads(panels_path.read_text(encoding="utf-8")).get("panel", [])
panel_locks = tomllib.loads(panel_locks_path.read_text(encoding="utf-8")).get("locks", {})
maps = tomllib.loads(maps_path.read_text(encoding="utf-8")).get("map", [])
map_locks = tomllib.loads(map_locks_path.read_text(encoding="utf-8")).get("locks", {})

enabled_panel = str(profile.get("reference_panel_id", "")).strip()
enabled_map = str(profile.get("reference_map_id", "")).strip()
species = str(profile.get("reference_species_id", "")).strip()
build = str(profile.get("reference_build_id", "")).strip()
cache_root = root / str(profile.get("reference_cache_root", "artifacts/vcf/reference_store")).strip()
redownload_policy = str(profile.get("reference_redownload_policy", "")).strip()
if redownload_policy != "never":
    errors.append("reference_redownload_policy must be 'never' (acquire once cache policy)")

panel_by_id = {str(p.get("id", "")).strip(): p for p in panels}
map_by_id = {str(m.get("id", "")).strip(): m for m in maps}
if enabled_panel not in panel_by_id:
    errors.append(f"enabled panel {enabled_panel} missing from panels.toml")
if enabled_map not in map_by_id:
    errors.append(f"enabled map {enabled_map} missing from maps.toml")

if enabled_panel and enabled_panel not in panel_locks:
    errors.append(f"enabled panel {enabled_panel} missing lock entry")
if enabled_map and enabled_map not in map_locks:
    errors.append(f"enabled map {enabled_map} missing lock entry")

def check_materialized(entry, lock, base):
    for f in lock.get("files", []):
        rel = str(f.get("path", "")).strip()
        exp = str(f.get("checksum_sha256", "")).strip()
        if not rel or not exp:
            errors.append(f"{entry}: lock file missing path/checksum")
            continue
        target = base / "raw" / rel
        if not target.exists():
            errors.append(f"{entry}: missing materialized file {target.relative_to(root)}")
            continue
        got = hashlib.sha256(target.read_bytes()).hexdigest()
        if got != exp:
            errors.append(f"{entry}: checksum mismatch for {target.relative_to(root)}")

if enabled_panel in panel_by_id and enabled_panel in panel_locks:
    p = panel_by_id[enabled_panel]
    if str(p.get("species_id", "")).strip() != species or str(p.get("build_id", "")).strip() != build:
        errors.append("enabled panel species/build does not match runtime profile")
    base = cache_root / "panels" / species / build / enabled_panel
    check_materialized(f"panel {enabled_panel}", panel_locks[enabled_panel], base)

if enabled_map in map_by_id and enabled_map in map_locks:
    m = map_by_id[enabled_map]
    if str(m.get("species_id", "")).strip() != species or str(m.get("build_id", "")).strip() != build:
        errors.append("enabled map species/build does not match runtime profile")
    base = cache_root / "maps" / species / build / enabled_map
    check_materialized(f"map {enabled_map}", map_locks[enabled_map], base)

for ex in example_paths:
    if not ex.exists():
        errors.append(f"missing example file {ex.relative_to(root)}")
        continue
    cfg = tomllib.loads(ex.read_text(encoding="utf-8"))
    panel_id = str(cfg.get("reference_panel_id", "")).strip()
    map_id = str(cfg.get("reference_map_id", "")).strip()
    if panel_id and panel_id not in panel_by_id:
        errors.append(f"{ex.relative_to(root)} references unknown panel {panel_id}")
    if map_id and map_id not in map_by_id:
        errors.append(f"{ex.relative_to(root)} references unknown map {map_id}")

if errors:
    print("vcf-reference-governance: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("vcf-reference-governance: OK")
PY
