#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  echo "Usage: scripts/checks/check-coverage-regimes-schema.sh"
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
path = root / "configs/runtime/coverage_regimes.toml"
raw = path.read_text(encoding="utf-8")
obj = tomllib.loads(raw)
errors = []

decision = obj.get("decision", {}).get("coverage_regime")
if not isinstance(decision, dict):
    errors.append("missing [decision.coverage_regime]")
else:
    thresholds = decision.get("thresholds", {})
    for key in ("gl_max_depth", "pseudohaploid_max_depth", "diploid_min_depth"):
        if key not in thresholds:
            errors.append(f"missing thresholds.{key}")
    outputs = decision.get("outputs", {})
    allowed = outputs.get("allowed_values")
    if allowed != ["gl", "pseudohaploid", "diploid"]:
        errors.append("outputs.allowed_values must be exactly [gl, pseudohaploid, diploid]")

if errors:
    print("coverage-regimes-schema: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("coverage-regimes-schema: OK")
PY
