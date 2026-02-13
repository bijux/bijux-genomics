#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

policy="$ROOT_DIR/configs/ci/tools/hpc_frontend_build_policy.toml"
[[ -f "$policy" ]] || { echo "hpc frontend policy: missing $policy" >&2; exit 1; }

build_script="$ROOT_DIR/scripts/containers/build-apptainer-all.sh"
frontend_script="$ROOT_DIR/scripts/containers/build-apptainer-hpc-frontend.sh"
smoke_script="$ROOT_DIR/scripts/containers/smoke-apptainer.sh"

for target in "$build_script" "$frontend_script" "$smoke_script"; do
  if ! rg -q "hpc_frontend_build_policy.toml|compute_hostname_regex|refusing.*compute node" "$target"; then
    echo "hpc frontend policy: missing compute-node refusal guard in $target" >&2
    exit 1
  fi
done

if ! rg -q "check-version-hash-pin.sh" "$frontend_script"; then
  echo "hpc frontend policy: frontend build script must enforce pinned versions via check-version-hash-pin.sh" >&2
  exit 1
fi

echo "hpc frontend policy enforcement: OK"
