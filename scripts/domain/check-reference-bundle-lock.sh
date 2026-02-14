#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
catalog="${ROOT_DIR}/configs/runtime/reference_bundles.toml"
lock="${ROOT_DIR}/configs/runtime/reference_bundles.lock.sha256"

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
