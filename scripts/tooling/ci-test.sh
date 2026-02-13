#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

nextest_config="${NEXTEST_CONFIG:---config-file configs/nextest/nextest.toml}"
test_features="${TEST_FEATURES:---all-features}"
nextest_profile="${NEXTEST_PROFILE:-ci}"
nextest_threads="${NEXTEST_TEST_THREADS:-8}"
nextest_no_tests="${NEXTEST_NO_TESTS:-pass}"
run_ignored="${RUN_IGNORED:---run-ignored all}"
nextest_expr="${NEXTEST_FAST_EXPR:-not test(/::slow__/)}"

./bin/isolate sh -ceu "
./bin/require-isolate >/dev/null
./scripts/checks/check-isolation-contract.sh
./scripts/checks/check-ssot-guardrails.sh
command -v cargo-nextest >/dev/null 2>&1 || { echo 'missing required tool: cargo-nextest'; echo 'install once: cargo install cargo-nextest --locked'; exit 1; }
chmod -R a-w assets
trap 'chmod -R u+w assets' EXIT
export TZ=UTC LC_ALL=C
export TEST_TARGET_DIR=\"\$ISO_ROOT/target-test\"
export COV_TARGET_DIR=\"\$ISO_ROOT/target-cov\"
export TEST_TMP_DIR=\"\$ISO_ROOT/tmp-test\"
export COV_TMP_DIR=\"\$ISO_ROOT/tmp-cov\"
export TEST_PROFRAW_DIR=\"\$ISO_ROOT/profraw-test\"
export COV_PROFRAW_DIR=\"\$ISO_ROOT/profraw-cov\"
export CARGO_TARGET_DIR=\"\$ISO_ROOT/target-test\"
if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER=\"\$(command -v sccache)\"; fi
cargo nextest run ${nextest_config} --workspace ${test_features} --profile ${nextest_profile} --test-threads ${nextest_threads} --no-tests ${nextest_no_tests} ${run_ignored} -E \"${nextest_expr}\"
./scripts/checks/check-isolation-contract.sh
"
