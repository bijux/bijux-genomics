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
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
planner_stage_ids = set()
for rel in (
    "configs/ci/stages/stages.toml",
    "configs/ci/stages/stages_vcf.toml",
    "configs/ci/stages/stages_vcf_downstream.toml",
):
    data = tomllib.loads((root / rel).read_text(encoding="utf-8"))
    for row in data.get("stages", []):
        sid = str(row.get("id", "")).strip()
        if sid:
            planner_stage_ids.add(sid)

errors = []
for stage_id in sorted(planner_stage_ids):
    domain = stage_id.split(".", 1)[0]
    fixture_dir = root / "domain" / domain / "fixtures" / stage_id
    if not fixture_dir.exists() or not any(p.is_file() for p in fixture_dir.rglob("*")):
        errors.append(
            f"{fixture_dir.relative_to(root)} missing fixture files for planner stage '{stage_id}'"
        )

if errors:
    print("planner fixture coverage check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("planner fixture coverage: OK")
PY
