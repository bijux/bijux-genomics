#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

./bin/isolate sh -ceu '
./bin/require-isolate >/dev/null
export CARGO_TARGET_DIR="$ISO_ROOT/target-ci-fast"
make _ci-fast
'
