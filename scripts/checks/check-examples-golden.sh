#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

errors=0
for ex in "$ROOT_DIR"/examples/* "$ROOT_DIR"/examples/*/*; do
  [[ -d "$ex" ]] || continue
  [[ -f "$ex/example.toml" ]] || continue
  [[ "$ex" == "$ROOT_DIR/examples/_template" ]] && continue
  rel="${ex#"$ROOT_DIR/"}"
  if [[ ! -f "$ex/golden/plan.json" ]]; then
    echo "examples golden: $rel missing golden/plan.json" >&2
    errors=1
  fi
  if [[ ! -f "$ex/golden/explain.json" ]]; then
    echo "examples golden: $rel missing golden/explain.json" >&2
    errors=1
  fi
  if [[ ! -f "$ex/golden/report.json" ]]; then
    echo "examples golden: $rel missing golden/report.json" >&2
    errors=1
  fi
  if ! python3 - "$ex" <<'PY'
from pathlib import Path
import json
import sys

ex = Path(sys.argv[1])
errors = []

def load(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"{path}: invalid JSON: {exc}")
        return None

plan = load(ex / "golden" / "plan.json")
explain = load(ex / "golden" / "explain.json")
report = load(ex / "golden" / "report.json")

if isinstance(plan, dict):
    for key in ("example_id",):
        if key not in plan:
            errors.append(f"{ex}/golden/plan.json missing key '{key}'")
    if "stages" in plan and not isinstance(plan["stages"], list):
        errors.append(f"{ex}/golden/plan.json key 'stages' must be a list")
if isinstance(explain, dict):
    for key in ("example_id",):
        if key not in explain:
            errors.append(f"{ex}/golden/explain.json missing key '{key}'")
if isinstance(report, dict):
    for key in ("example_id", "status"):
        if key not in report:
            errors.append(f"{ex}/golden/report.json missing key '{key}'")
if isinstance(plan, dict) and isinstance(explain, dict) and isinstance(report, dict):
    eid = plan.get("example_id")
    if explain.get("example_id") != eid or report.get("example_id") != eid:
        errors.append(f"{ex}/golden/*.json example_id values must match")

if errors:
    for err in errors:
        print(err, file=sys.stderr)
    raise SystemExit(1)
PY
  then
    errors=1
  fi
done

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi
echo "examples golden: OK"
