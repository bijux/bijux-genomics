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
crate_args="-p bijux-dna-engine -p bijux-dna-runner -p bijux-dna-runtime -p bijux-dna-api -p bijux-dna-stages-bam -p bijux-dna-stages-vcf"

require_artifact_env
CARGO_BUILD_JOBS="${cargo_build_jobs}" cargo clippy --all-targets --all-features ${crate_args} -- -D warnings
