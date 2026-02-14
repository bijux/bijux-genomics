#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SUMMARY_PATH="${1:-${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/hpc/frontend-reproducibility/${ISO_RUN_ID:-run}/summary.json}"
POLICY_TOML="${POLICY_TOML:-$ROOT_DIR/configs/ci/tools/apptainer_reproducibility_policy.toml}"

if [[ ! -f "$SUMMARY_PATH" ]]; then
  if [[ "${CI:-0}" == "1" ]]; then
    echo "frontend reproducibility check: missing summary in CI: $SUMMARY_PATH" >&2
    exit 1
  fi
  echo "frontend reproducibility check: SKIP (no summary at $SUMMARY_PATH)"
  exit 0
fi

python3 - "$SUMMARY_PATH" "$POLICY_TOML" <<'PY'
import json
import sys
from pathlib import Path
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

summary = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = tomllib.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
threshold = float(policy.get("confidence_min", 1.0))
require_all = bool(policy.get("require_all_tools_deterministic", True))

errors = []
confidence = float(summary.get("confidence", -1.0))
if confidence < threshold:
    errors.append(
        f"confidence below threshold: got {confidence:.4f}, need {threshold:.4f}"
    )

items = summary.get("items", [])
if require_all:
    bad = sorted(i.get("tool", "") for i in items if not i.get("deterministic", False))
    if bad:
        errors.append("non-deterministic tools: " + ", ".join(bad))

if errors:
    print("frontend reproducibility check: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("frontend reproducibility check: OK")
PY
