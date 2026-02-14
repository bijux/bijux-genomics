#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

BASE="${BASE:-${ISO_ROOT:-$ROOT_DIR/artifacts}/hpc/frontend-mini-e2e}"

python3 - "$BASE" <<'PY'
from pathlib import Path
import json
import sys

base = Path(sys.argv[1])
if not base.exists():
    print("frontend telemetry sanity: SKIP (no frontend-mini-e2e artifacts)")
    raise SystemExit(0)
runs = sorted([p for p in base.iterdir() if p.is_dir()])
if not runs:
    print("frontend telemetry sanity: SKIP (no run dirs)")
    raise SystemExit(0)
run = runs[-1]
summary = json.loads((run / "summary.json").read_text(encoding="utf-8"))
errors = []
for row in summary.get("examples", []):
    art = Path(str(row.get("artifact_dir", "")))
    metrics = art / "metrics.json"
    if not metrics.exists():
        errors.append(f"{art}: missing metrics.json")
        continue
    data = json.loads(metrics.read_text(encoding="utf-8"))
    for k in ("example_id", "collected_at", "status"):
        if k not in data:
            errors.append(f"{art}: metrics.json missing {k}")
    ex = str(data.get("example_id", ""))
    if ex.startswith("vcf_"):
        # downstream stages expected in bench suite for vcf mini profiles
        plan = art / "plan.json"
        if plan.exists():
            p = json.loads(plan.read_text(encoding="utf-8"))
            stages = set(str(s) for s in p.get("stages", []))
            needed = {"vcf.population_structure", "vcf.roh", "vcf.ibd", "vcf.demography"}
            if "downstream" in ex and not needed.issubset(stages):
                errors.append(f"{art}: plan missing required downstream stages for telemetry sanity")

if errors:
    print("frontend telemetry sanity: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print(f"frontend telemetry sanity: OK ({run.name})")
PY
