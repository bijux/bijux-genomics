#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

cargo_build_jobs="${CARGO_BUILD_JOBS:-8}"

require_artifact_env
CLIPPY_CONF_DIR='configs/rust' CARGO_BUILD_JOBS="${cargo_build_jobs}" cargo clippy --workspace --all-targets --all-features -- -D warnings
