#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import datetime as dt
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
dep_path = root / "configs/ci/registry/deprecations.toml"
rows = tomllib.loads(dep_path.read_text(encoding="utf-8")).get("deprecations", [])

def parse_date(s: str):
    return dt.date.fromisoformat(s)

# Build reference sets
required_tools = set()
for p in [
    root / "configs/ci/tools/required_tools.toml",
    root / "configs/ci/tools/required_tools_vcf.toml",
    root / "configs/ci/tools/required_tools_vcf_downstream.toml",
]:
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    required_tools |= {str(x) for x in data.get("required_tools", [])}

declared_stages = set()
for p in [
    root / "configs/ci/stages/stages.toml",
    root / "configs/ci/stages/stages_vcf.toml",
    root / "configs/ci/stages/stages_vcf_downstream.toml",
]:
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    declared_stages |= {str(x.get("id", "")).strip() for x in data.get("stages", []) if str(x.get("id", "")).strip()}

param_stage_ids = set()
for p in [
    root / "configs/ci/params/param_registry.toml",
    root / "configs/ci/params/param_registry_vcf.toml",
    root / "configs/ci/params/param_registry_downstream.toml",
]:
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    for row in data.get("entries", []) + data.get("params", []):
        sid = str(row.get("stage_id", "")).strip()
        if sid:
            param_stage_ids.add(sid)

errors: list[str] = []
today = dt.date.today()
for row in rows:
    tool = str(row.get("tool_id", "")).strip()
    stage = str(row.get("stage", "")).strip()
    removal_after = parse_date(str(row.get("removal_after", "")).strip())
    if today <= removal_after:
        continue
    if tool and tool in required_tools:
        errors.append(f"{dep_path.relative_to(root)}: deprecated tool '{tool}' past removal_after is still required")
    if stage and stage in declared_stages:
        errors.append(f"{dep_path.relative_to(root)}: deprecated stage '{stage}' past removal_after is still declared")
    if stage and stage in param_stage_ids:
        errors.append(f"{dep_path.relative_to(root)}: deprecated stage '{stage}' past removal_after still appears in param registries")

if errors:
    print("deprecations-enforcement: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("deprecations-enforcement: OK")
PY
