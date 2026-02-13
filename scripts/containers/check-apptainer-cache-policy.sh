#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

policy="$ROOT_DIR/configs/ci/tools/apptainer_cache_policy.toml"
[[ -f "$policy" ]] || { echo "apptainer cache policy: missing $policy" >&2; exit 1; }

for script in "$ROOT_DIR/scripts/containers/build-apptainer-all.sh" "$ROOT_DIR/scripts/containers/smoke-apptainer.sh"; do
  if ! rg -q "apptainer_cache_policy.toml|CACHE_POLICY_TOML" "$script"; then
    echo "apptainer cache policy: $script does not consume cache policy config" >&2
    exit 1
  fi
done

echo "apptainer cache policy: OK"

