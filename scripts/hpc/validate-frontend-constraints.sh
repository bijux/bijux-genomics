#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

POLICY_TOML="${POLICY_TOML:-$ROOT_DIR/configs/ci/tools/hpc_frontend_build_policy.toml}"
MIN_TMP_GB="${MIN_TMP_GB:-4}"
MIN_WORK_GB="${MIN_WORK_GB:-10}"
WORK_DIR="${WORK_DIR:-${ISO_ROOT:-$ROOT_DIR/artifacts}}"
dry_run=1
confirm=0

while [[ $# -gt 0 ]]; do
  case "${1:-}" in
    --dry-run) dry_run=1; confirm=0; shift ;;
    --confirm) dry_run=0; confirm=1; shift ;;
    --help|-h)
      cat <<'USAGE'
Usage: scripts/hpc/validate-frontend-constraints.sh [--dry-run|--confirm]
USAGE
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ "$dry_run" -eq 1 ]]; then
  echo "[dry-run] validate-frontend-constraints (pass --confirm to execute)"
  exit 0
fi

require_cmd python3
require_cmd df
require_cmd stat

host_name="$(hostname -f 2>/dev/null || hostname)"
set +e
python3 - "$POLICY_TOML" "$host_name" <<'PY'
import re
import os
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

cfg = tomllib.loads(open(sys.argv[1], "r", encoding="utf-8").read())
host = sys.argv[2]
compute_pat = str(cfg.get("compute_hostname_regex", "")).strip()
frontend_pat = str(cfg.get("frontend_hostname_regex", "")).strip()
if compute_pat and re.search(compute_pat, host):
    if "CI" in os.environ or os.environ.get("REQUIRE_FRONTEND") == "1":
        raise SystemExit(f"frontend constraints: refusing compute host '{host}'")
    print(f"frontend constraints: SKIP (compute host {host})")
    raise SystemExit(3)
if frontend_pat and not re.search(frontend_pat, host):
    if "CI" in os.environ or os.environ.get("REQUIRE_FRONTEND") == "1":
        raise SystemExit(f"frontend constraints: host '{host}' does not match frontend pattern")
    print(f"frontend constraints: SKIP (host {host} not frontend)")
    raise SystemExit(3)
print(f"frontend constraints: host policy OK ({host})")
PY
rc=$?
set -e
if [[ "$rc" -eq 3 ]]; then
  exit 0
fi
if [[ "$rc" -ne 0 ]]; then
  exit "$rc"
fi

tmp_kb="$(df -Pk /tmp | awk 'NR==2{print $4}')"
work_kb="$(df -Pk "$WORK_DIR" | awk 'NR==2{print $4}')"
tmp_gb=$((tmp_kb / 1024 / 1024))
work_gb=$((work_kb / 1024 / 1024))
if (( tmp_gb < MIN_TMP_GB )); then
  echo "frontend constraints: /tmp free ${tmp_gb}GB < required ${MIN_TMP_GB}GB" >&2
  exit 1
fi
if (( work_gb < MIN_WORK_GB )); then
  echo "frontend constraints: work dir free ${work_gb}GB < required ${MIN_WORK_GB}GB ($WORK_DIR)" >&2
  exit 1
fi

test_dir="$WORK_DIR/hpc-frontend-constraints.$$"
mkdir -p "$test_dir"
touch "$test_dir/.write_test"
chmod 700 "$test_dir"
rm -f "$test_dir/.write_test"
rmdir "$test_dir"

if command -v module >/dev/null 2>&1; then
  if ! module avail >/dev/null 2>&1; then
    echo "frontend constraints: module command exists but module avail failed" >&2
    exit 1
  fi
  module_state="available"
else
  module_state="not_used"
fi

echo "frontend constraints: OK (tmp=${tmp_gb}GB work=${work_gb}GB modules=${module_state})"
