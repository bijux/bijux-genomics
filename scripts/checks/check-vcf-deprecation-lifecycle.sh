#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
python3 - "$ROOT_DIR" <<'PY'
import sys
from pathlib import Path
try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore[no-redef]
root = Path(sys.argv[1])
knobs = tomllib.loads((root / "configs/vcf/deprecations/knobs.toml").read_text(encoding="utf-8"))
panels = tomllib.loads((root / "configs/vcf/deprecations/panels.toml").read_text(encoding="utf-8"))
param = tomllib.loads((root / "configs/ci/params/param_registry_downstream.toml").read_text(encoding="utf-8"))
panel_catalog = tomllib.loads((root / "configs/vcf/panels/panels.toml").read_text(encoding="utf-8"))
errors = []

allowed = {"warn", "fail", "remove"}
entries = param.get("entries", [])
allowed_by_stage = {str(e.get("stage_id", "")): set(map(str, e.get("params", []))) for e in entries}
for row in knobs.get("deprecation", []):
    stage = str(row.get("stage_id", "")).strip()
    knob = str(row.get("knob", "")).strip()
    phase = str(row.get("phase", "")).strip()
    if phase not in allowed:
        errors.append(f"knob deprecation {stage}:{knob} has invalid phase '{phase}'")
    if phase == "remove" and knob in allowed_by_stage.get(stage, set()):
        errors.append(f"removed knob still present in param registry: {stage}:{knob}")

catalog_panel_ids = {str(p.get("id", "")).strip() for p in panel_catalog.get("panel", [])}
for row in panels.get("deprecation", []):
    panel_id = str(row.get("panel_id", "")).strip()
    phase = str(row.get("phase", "")).strip()
    if phase not in allowed:
        errors.append(f"panel deprecation {panel_id} has invalid phase '{phase}'")
    if phase == "remove" and panel_id in catalog_panel_ids:
        errors.append(f"removed panel still present in catalog: {panel_id}")

if errors:
    print("vcf deprecation lifecycle: FAILED", file=sys.stderr)
    for e in errors:
        print(f" - {e}", file=sys.stderr)
    raise SystemExit(1)
print("vcf deprecation lifecycle: OK")
PY
