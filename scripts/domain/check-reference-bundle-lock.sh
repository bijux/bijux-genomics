#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

catalog="${ROOT_DIR}/configs/runtime/reference_bundles.toml"
lock="${ROOT_DIR}/configs/runtime/reference_bundles_lock.sha256"
materialization_lock_json="${ROOT_DIR}/configs/runtime/references/locks/lock.json"
materialization_lock_sha="${ROOT_DIR}/configs/runtime/references/locks/lock.json.sha256"

if [[ ! -f "$catalog" ]]; then
  echo "reference bundle lock check: missing ${catalog}" >&2
  exit 1
fi
if [[ ! -f "$lock" ]]; then
  echo "reference bundle lock check: missing ${lock}" >&2
  exit 1
fi

expected="$(python3 - <<'PY' "$catalog"
import hashlib, pathlib, sys
path = pathlib.Path(sys.argv[1])
print(hashlib.sha256(path.read_bytes()).hexdigest())
PY
)"

actual="$(tr -d '[:space:]' < "$lock")"
if [[ "$expected" != "$actual" ]]; then
  echo "reference bundle lock drift: ${lock} is stale; update it after bundle changes" >&2
  echo "expected=${expected}" >&2
  echo "actual=${actual}" >&2
  exit 1
fi

echo "reference bundle lock: OK"

if [[ -f "$materialization_lock_json" || -f "$materialization_lock_sha" ]]; then
  if [[ ! -f "$materialization_lock_json" ]]; then
    echo "reference materialization lock check: missing ${materialization_lock_json}" >&2
    exit 1
  fi
  if [[ ! -f "$materialization_lock_sha" ]]; then
    echo "reference materialization lock check: missing ${materialization_lock_sha}" >&2
    exit 1
  fi

  mat_expected="$(python3 - <<'PY' "$materialization_lock_json"
import hashlib, pathlib, sys
path = pathlib.Path(sys.argv[1])
print(hashlib.sha256(path.read_bytes()).hexdigest())
PY
)"
  mat_actual="$(awk '{print $1}' "$materialization_lock_sha" | tr -d '[:space:]')"
  if [[ "$mat_expected" != "$mat_actual" ]]; then
    echo "reference materialization lock drift: ${materialization_lock_sha} is stale" >&2
    echo "expected=${mat_expected}" >&2
    echo "actual=${mat_actual}" >&2
    exit 1
  fi
  echo "reference materialization lock: OK"
fi
