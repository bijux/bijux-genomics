#!/usr/bin/env bash
set -euo pipefail

command_name="${1:-}"
if [ -z "${command_name}" ]; then
  echo "usage: rust_gate.sh <fmt|lint|audit|test|test-slow|test-all|coverage>" >&2
  exit 2
fi
shift || true

workspace_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${workspace_root}"

rs_artifact_root="${RS_ARTIFACT_ROOT:-artifacts/rust}"
rs_artifact_root="$(cd "$(dirname "${rs_artifact_root}")" && pwd)/$(basename "${rs_artifact_root}")"
rs_run_id="${RS_RUN_ID:-local}"

rs_target_dir="${RS_TARGET_DIR:-${rs_artifact_root}/target}"
rs_nextest_cache_dir="${RS_NEXTEST_CACHE_DIR:-${rs_target_dir}/nextest}"
rs_nextest_config_home="${RS_NEXTEST_CONFIG_HOME:-${rs_artifact_root}/nextest/config}"
rs_profraw_dir="${RS_PROFRAW_DIR:-${rs_artifact_root}/coverage/profraw}"
rs_llvm_profile_file="${RS_LLVM_PROFILE_FILE:-${rs_profraw_dir}/default_%m_%p.profraw}"
rs_coverage_target_dir="${RS_COVERAGE_TARGET_DIR:-${rs_artifact_root}/coverage/target}"

rs_fmt_report="${RS_FMT_REPORT:-${rs_artifact_root}/fmt/${rs_run_id}/report.txt}"
rs_lint_report="${RS_LINT_REPORT:-${rs_artifact_root}/lint/${rs_run_id}/report.txt}"
rs_test_report="${RS_TEST_REPORT:-${rs_artifact_root}/test/${rs_run_id}/nextest.log}"
rs_test_slow_report="${RS_TEST_SLOW_REPORT:-${rs_artifact_root}/test/${rs_run_id}/nextest-slow.log}"
rs_test_all_report="${RS_TEST_ALL_REPORT:-${rs_artifact_root}/test/${rs_run_id}/nextest-all.log}"
rs_audit_report="${RS_AUDIT_REPORT:-${rs_artifact_root}/audit/${rs_run_id}/report.txt}"
rs_coverage_dir="${RS_COVERAGE_DIR:-${rs_artifact_root}/coverage/${rs_run_id}}"
rs_lcov_file="${RS_LCOV_FILE:-${rs_coverage_dir}/lcov.info}"
rs_coverage_test_report="${RS_COVERAGE_TEST_REPORT:-${rs_coverage_dir}/nextest.log}"
rs_coverage_summary_report="${RS_COVERAGE_SUMMARY_REPORT:-${rs_coverage_dir}/summary.txt}"

cargo_term_progress_when="${CARGO_TERM_PROGRESS_WHEN:-always}"
cargo_term_progress_width="${CARGO_TERM_PROGRESS_WIDTH:-120}"
cargo_term_verbose="${CARGO_TERM_VERBOSE:-false}"
cargo_term_color="${CARGO_TERM_COLOR:-always}"

nextest_config_file="${NEXTEST_CONFIG_FILE:-configs/rust/nextest.toml}"
nextest_profile_fast="${NEXTEST_PROFILE_FAST:-fast-unit}"
nextest_profile_slow="${NEXTEST_PROFILE_SLOW:-slow-integration}"
nextest_profile_all="${NEXTEST_PROFILE_ALL:-full}"
nextest_status_level="${NEXTEST_STATUS_LEVEL:-all}"
nextest_final_status_level="${NEXTEST_FINAL_STATUS_LEVEL:-all}"
nextest_fast_expr="${NEXTEST_FAST_EXPR:-not test(/::slow__/)}"
nextest_slow_expr="${NEXTEST_SLOW_EXPR:-test(/::slow__/)}"
rs_clippy_excludes="${RS_CLIPPY_EXCLUDES:-}"

default_fast_test_runner() {
  case "$(uname -s)" in
    Darwin)
      printf '%s' "package-nextest"
      ;;
    *)
      printf '%s' "nextest"
      ;;
  esac
}

default_nextest_threads() {
  case "$(uname -s)" in
    Darwin)
      printf '%s' "1"
      ;;
    *)
      printf '%s' ""
      ;;
  esac
}

fast_test_runner="${RS_FAST_TEST_RUNNER:-$(default_fast_test_runner)}"
nextest_threads="${NEXTEST_THREADS:-$(default_nextest_threads)}"

mkdir -p \
  "${rs_artifact_root}" \
  "${rs_target_dir}" \
  "${rs_nextest_cache_dir}" \
  "${rs_nextest_config_home}" \
  "${rs_profraw_dir}" \
  "${rs_coverage_target_dir}" \
  "${rs_coverage_dir}"

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 is required but not installed" >&2
    exit 1
  fi
}

strip_ansi() {
  perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g'
}

print_nextest_summary() {
  local report_path="$1"
  local summary_line
  summary_line="$(strip_ansi < "${report_path}" | grep 'Summary \[' | tail -n 1 || true)"
  printf '\033[1;36m%s\033[0m %s\n' "nextest-summary:" "${summary_line:-unavailable}"
}

prepare_common_env() {
  export TZ="UTC"
  export LC_ALL="C"
  export ARTIFACT_ROOT="${ARTIFACT_ROOT:-artifacts}"
  export ISO_ROOT="${ISO_ROOT:-$(cd "${ARTIFACT_ROOT}" && pwd 2>/dev/null || printf '%s/%s' "${workspace_root}" "${ARTIFACT_ROOT}")}"
  export CARGO_HOME="${CARGO_HOME:-${workspace_root}/artifacts/cargo/home}"
  export TMPDIR="${TMPDIR:-${workspace_root}/artifacts/tmp}"
  export TMP="${TMP:-${TMPDIR}}"
  export TEMP="${TEMP:-${TMPDIR}}"
  mkdir -p "${CARGO_HOME}" "${TMPDIR}"
  export CARGO_TERM_COLOR="${cargo_term_color}"
  export CARGO_TERM_PROGRESS_WHEN="${cargo_term_progress_when}"
  export CARGO_TERM_PROGRESS_WIDTH="${cargo_term_progress_width}"
  export CARGO_TERM_VERBOSE="${cargo_term_verbose}"
}

run_logged() {
  local report_path="$1"
  shift
  mkdir -p "$(dirname "${report_path}")"
  set -o pipefail
  "$@" 2>&1 | tee "${report_path}"
}

run_nextest() {
  local report_path="$1"
  local target_dir="$2"
  shift 2
  local nextest_args=("$@")

  if [ -n "${nextest_threads}" ]; then
    nextest_args+=(-j "${nextest_threads}")
  fi

  mkdir -p "$(dirname "${report_path}")" "${rs_profraw_dir}" "${rs_nextest_config_home}" "${rs_nextest_cache_dir}"
  local status=0
  set +e
  set -o pipefail
  env \
    CARGO_TARGET_DIR="${target_dir}" \
    NEXTEST_CACHE_DIR="${rs_nextest_cache_dir}" \
    XDG_CONFIG_HOME="${rs_nextest_config_home}" \
    LLVM_PROFILE_FILE="${rs_llvm_profile_file}" \
    "${nextest_args[@]}" --target-dir "${target_dir}" 2>&1 | tee "${report_path}"
  status=$?
  set -e
  print_nextest_summary "${report_path}"
  return "${status}"
}

workspace_package_names() {
  cargo metadata --format-version 1 --no-deps | perl -MJSON::PP -e '
    my $metadata = decode_json(do { local $/; <STDIN> });
    my %members = map { $_ => 1 } @{$metadata->{workspace_members}};
    for my $package (sort { $a->{name} cmp $b->{name} } @{$metadata->{packages}}) {
      print "$package->{name}\n" if $members{$package->{id}};
    }
  '
}

run_nextest_by_package() {
  local report_root="$1"
  local target_dir="$2"
  shift 2
  while IFS= read -r package_name; do
    [ -n "${package_name}" ] || continue
    local package_report="${report_root%.log}--${package_name}.log"
    printf '%s\n' "run: cargo nextest run -p ${package_name} --all-features --profile ${nextest_profile_fast} -E ${nextest_fast_expr}"
    run_nextest "${package_report}" "${target_dir}" "$@" -p "${package_name}"
  done < <(workspace_package_names)
}

run_cargo_test() {
  local report_path="$1"
  local target_dir="$2"
  shift 2
  mkdir -p "$(dirname "${report_path}")" "${rs_profraw_dir}"
  run_logged "${report_path}" env \
    CARGO_TARGET_DIR="${target_dir}" \
    LLVM_PROFILE_FILE="${rs_llvm_profile_file}" \
    "$@"
}

audit_ignore_args=()
if [ -f "audit-allowlist.toml" ]; then
  while IFS= read -r advisory_id; do
    [ -n "${advisory_id}" ] || continue
    audit_ignore_args+=(--ignore "${advisory_id}")
  done < <(rg -o --no-line-number 'RUSTSEC-[0-9]{4}-[0-9]{4}' audit-allowlist.toml | sort -u)
fi

prepare_common_env

case "${command_name}" in
  fmt)
    printf '%s\n' "run: cargo fmt --all -- --check"
    run_logged "${rs_fmt_report}" env \
      CARGO_TARGET_DIR="${rs_target_dir}" \
      cargo fmt --all -- --check
    ;;
  lint)
    printf '%s\n' "run: cargo clippy --workspace --all-targets --all-features --locked -- -D warnings"
    clippy_args=(
      cargo clippy
      --workspace
      --all-targets
      --all-features
      --locked
    )
    if [ -n "${rs_clippy_excludes}" ]; then
      read -r -a excluded_crates <<< "${rs_clippy_excludes}"
      for excluded_crate in "${excluded_crates[@]}"; do
        clippy_args+=(--exclude "${excluded_crate}")
      done
    fi
    clippy_args+=(-- -D warnings)
    run_logged "${rs_lint_report}" env \
      CLIPPY_CONF_DIR="configs/rust" \
      CARGO_TARGET_DIR="${rs_target_dir}" \
      "${clippy_args[@]}"
    ;;
  audit)
    require_tool cargo-deny
    require_tool cargo-audit
    mkdir -p "$(dirname "${rs_audit_report}")"
    set -o pipefail
    governance_status=0
    deny_status=0
    audit_status=0
    {
      echo "run: cargo run -q -p bijux-dna-dev -- checks run check-audit-allowlist"
      CARGO_TARGET_DIR="${rs_target_dir}" cargo run -q -p bijux-dna-dev -- checks run check-audit-allowlist || governance_status=$?
      echo
      echo "run: cargo run -q -p bijux-dna-dev -- checks run check-deny-policy-deviations"
      CARGO_TARGET_DIR="${rs_target_dir}" cargo run -q -p bijux-dna-dev -- checks run check-deny-policy-deviations || governance_status=$?
      echo
      echo "run: cargo deny check bans licenses sources --config configs/rust/deny.toml"
      CARGO_TARGET_DIR="${rs_target_dir}" cargo deny check bans licenses sources --config configs/rust/deny.toml || deny_status=$?
      echo
      echo "run: cargo audit ${audit_ignore_args[*]:-}"
      CARGO_TARGET_DIR="${rs_target_dir}" cargo audit "${audit_ignore_args[@]}" || audit_status=$?
    } 2>&1 | tee "${rs_audit_report}"
    test "${governance_status}" -eq 0
    test "${deny_status}" -eq 0
    test "${audit_status}" -eq 0
    ;;
  test)
    if [ "${fast_test_runner}" = "package-nextest" ]; then
      require_tool cargo-nextest
      run_nextest_by_package "${rs_test_report}" "${rs_target_dir}" cargo nextest run \
        --all-features \
        --config-file "${nextest_config_file}" \
        --profile "${nextest_profile_fast}" \
        --status-level "${nextest_status_level}" \
        --final-status-level "${nextest_final_status_level}" \
        -E "${nextest_fast_expr}"
    elif [ "${fast_test_runner}" = "cargo" ]; then
      printf '%s\n' "run: cargo test --workspace --all-features --no-fail-fast -- --skip ::slow__"
      run_cargo_test "${rs_test_report}" "${rs_target_dir}" cargo test \
        --workspace \
        --all-features \
        --no-fail-fast \
        -- \
        --skip "::slow__"
    else
      require_tool cargo-nextest
      printf '%s\n' "run: cargo nextest run --workspace --all-features --profile ${nextest_profile_fast} -E ${nextest_fast_expr}"
      run_nextest "${rs_test_report}" "${rs_target_dir}" cargo nextest run \
        --workspace \
        --all-features \
        --config-file "${nextest_config_file}" \
        --profile "${nextest_profile_fast}" \
        --status-level "${nextest_status_level}" \
        --final-status-level "${nextest_final_status_level}" \
        -E "${nextest_fast_expr}"
    fi
    ;;
  test-slow)
    require_tool cargo-nextest
    printf '%s\n' "run: cargo nextest run --workspace --all-features --profile ${nextest_profile_slow} -E ${nextest_slow_expr}"
    run_nextest "${rs_test_slow_report}" "${rs_target_dir}" cargo nextest run \
      --workspace \
      --all-features \
      --config-file "${nextest_config_file}" \
      --profile "${nextest_profile_slow}" \
      --status-level "${nextest_status_level}" \
      --final-status-level "${nextest_final_status_level}" \
      -E "${nextest_slow_expr}"
    ;;
  test-all)
    require_tool cargo-nextest
    printf '%s\n' "run: cargo nextest run --workspace --all-features --run-ignored all --profile ${nextest_profile_all}"
    run_nextest "${rs_test_all_report}" "${rs_target_dir}" cargo nextest run \
      --workspace \
      --all-features \
      --run-ignored all \
      --retries 0 \
      --config-file "${nextest_config_file}" \
      --profile "${nextest_profile_all}" \
      --status-level "${nextest_status_level}" \
      --final-status-level "${nextest_final_status_level}"
    ;;
  coverage)
    require_tool cargo-nextest
    require_tool cargo-llvm-cov
    mkdir -p "${rs_coverage_dir}" "${rs_profraw_dir}" "${rs_nextest_config_home}" "${rs_nextest_cache_dir}"
    printf '%s\n' "run: cargo llvm-cov nextest --workspace --all-features --run-ignored all --profile ${nextest_profile_all}"
    run_nextest "${rs_coverage_test_report}" "${rs_coverage_target_dir}" env \
      CARGO_LLVM_COV_TARGET_DIR="${rs_coverage_target_dir}" \
      cargo llvm-cov nextest \
      --workspace \
      --all-features \
      --run-ignored all \
      --retries 0 \
      --config-file "${nextest_config_file}" \
      --profile "${nextest_profile_all}" \
      --status-level "${nextest_status_level}" \
      --final-status-level "${nextest_final_status_level}" \
      --lcov \
      --output-path "${rs_lcov_file}"
    run_logged "${rs_coverage_summary_report}" env \
      CARGO_TARGET_DIR="${rs_coverage_target_dir}" \
      CARGO_LLVM_COV_TARGET_DIR="${rs_coverage_target_dir}" \
      cargo llvm-cov report --summary-only
    total_line="$(strip_ansi < "${rs_coverage_summary_report}" | grep '^TOTAL' | tail -n 1 || true)"
    printf '\033[1;36m%s\033[0m %s\n' "coverage-summary:" "${total_line:-unavailable}"
    printf '\033[1;36m%s\033[0m %s\n' "coverage-lcov:" "${rs_lcov_file}"
    printf '\033[1;36m%s\033[0m %s\n' "coverage-report:" "${rs_coverage_summary_report}"
    ;;
  *)
    echo "unknown command: ${command_name}" >&2
    exit 2
    ;;
esac
