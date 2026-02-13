#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SUMMARY_PATH="${1:-${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/hpc/frontend-security/${ISO_RUN_ID:-run}/security_summary.json}"
POLICY_TOML="${POLICY_TOML:-$ROOT_DIR/configs/ci/tools/apptainer_security_policy.toml}"

if [[ ! -f "$SUMMARY_PATH" ]]; then
  if [[ "${CI:-0}" == "1" ]]; then
    echo "frontend security check: missing summary in CI: $SUMMARY_PATH" >&2
    exit 1
  fi
  echo "frontend security check: SKIP (no summary at $SUMMARY_PATH)"
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
fail_on_critical = bool(policy.get("fail_on_unallowlisted_critical", True))

errors = []
if not summary.get("items"):
    errors.append("no SBOM/SIF items recorded")
if summary.get("license_mismatches"):
    errors.append("license mismatches present")
if fail_on_critical and summary.get("critical_unallowlisted"):
    errors.append("unallowlisted critical CVEs present")
if not summary.get("ok", False):
    errors.append("summary status is fail")

if errors:
    print("frontend security check: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("frontend security check: OK")
PY
