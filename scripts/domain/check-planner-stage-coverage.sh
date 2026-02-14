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
import re
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
for stage_path in sorted((root / "domain").glob("*/stages/*.yaml")):
    if stage_path.name == "_schema.yaml":
        continue
    text = stage_path.read_text(encoding="utf-8")
    sid_m = re.search(r'^stage_id:\s*"?([^"\n#]+)"?\s*$', text, flags=re.MULTILINE)
    status_m = re.search(r'^status:\s*"?([^"\n#]+)"?\s*$', text, flags=re.MULTILINE)
    if not sid_m:
        continue
    stage_id = sid_m.group(1).strip()
    status = status_m.group(1).strip() if status_m else ""
    if status != "supported":
        continue
    if stage_id not in planner_stage_ids:
        errors.append(
            f"{stage_path.relative_to(root)}: supported stage '{stage_id}' missing planner coverage in configs/ci/stages/*.toml"
        )

if errors:
    print("planner stage coverage check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("planner stage coverage: OK")
PY
