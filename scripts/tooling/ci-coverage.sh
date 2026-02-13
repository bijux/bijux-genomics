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
runner_cfg="${ROOT_DIR}/configs/coverage/runner.toml"
read_cfg="$(python3 - "$runner_cfg" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
cfg = tomllib.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(cfg.get("nextest_profile", "ci"))
print(cfg.get("test_threads", 1))
print("--run-ignored all" if bool(cfg.get("run_ignored", True)) else "")
print("--no-cfg-coverage" if bool(cfg.get("no_cfg_coverage", True)) else "")
PY
)"
cfg_profile="$(printf '%s\n' "$read_cfg" | sed -n '1p')"
cfg_threads="$(printf '%s\n' "$read_cfg" | sed -n '2p')"
cfg_run_ignored="$(printf '%s\n' "$read_cfg" | sed -n '3p')"
cfg_no_cfg_cov="$(printf '%s\n' "$read_cfg" | sed -n '4p')"
nextest_profile="${NEXTEST_PROFILE:-$cfg_profile}"
nextest_threads="${NEXTEST_TEST_THREADS:-$cfg_threads}"
run_ignored="${RUN_IGNORED:-$cfg_run_ignored}"
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
cargo llvm-cov nextest --no-report ${cfg_no_cfg_cov} ${nextest_config} --workspace ${test_features} --profile ${nextest_profile} --test-threads ${nextest_threads} ${run_ignored}
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
