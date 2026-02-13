#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

./bin/isolate sh -ceu '
./bin/require-isolate >/dev/null
./scripts/run.sh checks check-audit-allowlist
command -v cargo-deny >/dev/null 2>&1 || { echo "missing required tool: cargo-deny"; echo "install once: cargo install cargo-deny --locked"; exit 1; }
cargo deny check
'
