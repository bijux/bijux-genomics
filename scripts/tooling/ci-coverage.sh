#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

nextest_config="${NEXTEST_CONFIG:---config-file configs/nextest/nextest.toml}"
test_features="${TEST_FEATURES:---all-features}"
nextest_profile="${NEXTEST_PROFILE:-ci}"
nextest_threads="${NEXTEST_TEST_THREADS:-8}"
run_ignored="${RUN_IGNORED:---run-ignored all}"
coverage_out="${COVERAGE_OUT:-coverage.json}"
coverage_baseline="${COVERAGE_BASELINE:-artifacts/coverage/baseline.json}"
coverage_thresholds="${COVERAGE_THRESHOLDS:-configs/coverage/thresholds.toml}"

./bin/isolate sh -ceu "
./bin/require-isolate >/dev/null
command -v cargo-llvm-cov >/dev/null 2>&1 || { echo 'missing required tool: cargo-llvm-cov'; echo 'install once: cargo install cargo-llvm-cov --locked'; exit 1; }
command -v cargo-nextest >/dev/null 2>&1 || { echo 'missing required tool: cargo-nextest'; echo 'install once: cargo install cargo-nextest --locked'; exit 1; }
export TZ=UTC LC_ALL=C
export TEST_TARGET_DIR=\"\$ISO_ROOT/target-test\"
export COV_TARGET_DIR=\"\$ISO_ROOT/target-cov\"
export TEST_TMP_DIR=\"\$ISO_ROOT/tmp-test\"
export COV_TMP_DIR=\"\$ISO_ROOT/tmp-cov\"
export TEST_PROFRAW_DIR=\"\$ISO_ROOT/profraw-test\"
export COV_PROFRAW_DIR=\"\$ISO_ROOT/profraw-cov\"
export CARGO_TARGET_DIR=\"\$ISO_ROOT/target-test\"
if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER=\"\$(command -v sccache)\"; fi
cargo llvm-cov clean
rm -rf \"\$ISO_ROOT/coverage\"
mkdir -p \"\$ISO_ROOT/coverage\"
cargo llvm-cov nextest --no-report --no-cfg-coverage ${nextest_config} --workspace ${test_features} --profile ${nextest_profile} --test-threads ${nextest_threads} ${run_ignored}
cargo llvm-cov report --json --output-path \"\$ISO_ROOT/coverage/${coverage_out}\"
cargo llvm-cov report --html --output-dir \"\$ISO_ROOT/coverage\"
test -f \"\$ISO_ROOT/coverage/${coverage_out}\"
test -f \"\$ISO_ROOT/coverage/index.html\"
if [ -f \"${coverage_baseline}\" ]; then
  python3 scripts/tooling/coverage_summary.sh \"\$ISO_ROOT/coverage/${coverage_out}\" --baseline \"${coverage_baseline}\" --check-thresholds \"${coverage_thresholds}\"
else
  python3 scripts/tooling/coverage_summary.sh \"\$ISO_ROOT/coverage/${coverage_out}\" --check-thresholds \"${coverage_thresholds}\"
fi
"
