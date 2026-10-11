#!/usr/bin/env bash
set -euo pipefail

if ((BASH_VERSINFO[0] < 5 || (BASH_VERSINFO[0] == 5 && BASH_VERSINFO[1] < 1))); then
  echo "GitHub workflow gate supervision requires Bash 5.1 or newer for wait -n -p" >&2
  exit 1
fi

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "required tool is unavailable: $1" >&2
    exit 1
  fi
}

require_tool git
require_tool python3
require_tool ruby
require_tool cargo
require_tool sccache

repo_root="$(git rev-parse --show-toplevel)"
artifact_root_input="${ARTIFACT_ROOT:-${repo_root}/artifacts}"
if [[ "${artifact_root_input}" != /* ]]; then
  artifact_root_input="${repo_root}/${artifact_root_input}"
fi
mkdir -p "${artifact_root_input}"
artifact_root="$(cd "${artifact_root_input}" && pwd -P)"
cargo_target_dir="${artifact_root}/target"
cargo_home="${artifact_root}/cargo/home"
mkdir -p "${cargo_target_dir}" "${cargo_home}"
export CARGO_TARGET_DIR="${cargo_target_dir}"
export CARGO_HOME="${cargo_home}"
export RS_TARGET_DIR="${cargo_target_dir}"
export RS_CARGO_HOME="${cargo_home}"

artifact_dir="${GITHUB_WORKFLOW_ARTIFACT_DIR:-${artifact_root}/github-all}"
make_bin="${GITHUB_WORKFLOW_MAKE_BIN:-make}"
summary_file="${artifact_dir}/summary.tsv"
metrics_dir="${artifact_dir}/build-authority"
metrics_after="${metrics_dir}/sccache-after.json"
workflow_file="${repo_root}/.github/workflows/ci.yml"
nextest_config_file="${NEXTEST_CONFIG_FILE:-${repo_root}/configs/rust/nextest.toml}"
nextest_profile="${NEXTEST_PROFILE_FAST:-fast-unit}"

require_tool "${make_bin}"

publish_sccache_metrics() {
  local label="$1"
  local output="${metrics_dir}/sccache-${label}.json"
  local pending="${output}.pending"
  sccache --show-stats --stats-format=json >"${pending}" || return $?
  mv "${pending}" "${output}"
}

# shellcheck disable=SC2329  # Invoked indirectly by the EXIT trap.
publish_final_metrics() {
  local status="$?"
  trap - EXIT
  if ! publish_sccache_metrics after; then
    echo "failed to publish final sccache metrics: ${metrics_after}" >&2
    status=1
  fi
  exit "${status}"
}

if [[ ! -f "${workflow_file}" ]]; then
  echo "GitHub CI workflow is unavailable: ${workflow_file}" >&2
  exit 1
fi

gate_listing="$(python3 "${repo_root}/makes/bin/github_workflow_gates.py" "${workflow_file}")"
mapfile -t admitted_gates <<<"${gate_listing}"
preparation_mode="${admitted_gates[0]}"
gate_names=()
gate_targets=()
for gate_target in "${admitted_gates[@]:1}"; do
  gate_name="${gate_target#_}"
  gate_name="${gate_name//_/-}"
  gate_names+=("${gate_name}")
  gate_targets+=("${gate_target}")
done

mkdir -p "${artifact_dir}" "${metrics_dir}" "${artifact_dir}/targets"
publish_sccache_metrics before
trap publish_final_metrics EXIT
rm -f \
  "${artifact_dir}"/*.log \
  "${summary_file}"
printf 'gate\ttarget\texit_code\tlog\n' >"${summary_file}"

workspace_compile_log="${artifact_dir}/workspace-compile.log"
shared_dev_bin="${cargo_target_dir}/debug/bijux-dna-dev"
if [[ "${preparation_mode}" == "test-preparation" ]]; then
  require_tool cargo-nextest
  printf 'preparing shared workspace binaries\nlog: %s\n' "${workspace_compile_log}"
  if (
    "${make_bin}" --no-print-directory _dev-dna-bin || exit $?
    cargo nextest list \
      --workspace \
      --all-features \
      --locked \
      --config-file "${nextest_config_file}" \
      --profile "${nextest_profile}" \
      --target-dir "${cargo_target_dir}" \
      >/dev/null
  ) >"${workspace_compile_log}" 2>&1; then
    printf 'prepared shared workspace binaries\n'
  else
    compile_status=$?
    printf 'workspace binary preparation failed (exit %s)\n' "${compile_status}" >&2
    tail -n 80 "${workspace_compile_log}" >&2
    exit "${compile_status}"
  fi
fi

pids=()
declare -A gate_index_by_pid=()

# shellcheck disable=SC2329  # Invoked indirectly by the signal trap.
terminate_gates() {
  for pid in "${pids[@]:-}"; do
    kill "${pid}" 2>/dev/null || true
  done
}
trap terminate_gates INT TERM

for index in "${!gate_names[@]}"; do
  gate_name="${gate_names[${index}]}"
  gate_target="${gate_targets[${index}]}"
  gate_log="${artifact_dir}/${gate_name}.log"
  gate_target_dir="${artifact_dir}/targets/${gate_name}"
  mkdir -p "${gate_target_dir}"
  printf 'started gate: %s (make %s)\nlog: %s\n' \
    "${gate_name}" \
    "${gate_target}" \
    "${gate_log}"
  (
    if [[ "${preparation_mode}" == "test-preparation" ]]; then
      CARGO_TARGET_DIR="${gate_target_dir}" \
        RS_TARGET_DIR="${gate_target_dir}" \
        DEV_DNA_BIN="${shared_dev_bin}" \
        "${make_bin}" --no-print-directory "${gate_target}" >"${gate_log}" 2>&1
    else
      CARGO_TARGET_DIR="${gate_target_dir}" \
        RS_TARGET_DIR="${gate_target_dir}" \
        "${make_bin}" --no-print-directory "${gate_target}" >"${gate_log}" 2>&1
    fi
  ) &
  gate_pid="$!"
  pids+=("${gate_pid}")
  gate_index_by_pid["${gate_pid}"]="${index}"
done

overall_status=0
exit_codes=()
remaining_pids=("${pids[@]}")
while [[ "${#remaining_pids[@]}" -gt 0 ]]; do
  completed_pid=""
  set +e
  wait -n -p completed_pid "${remaining_pids[@]}"
  exit_code=$?
  set -e
  if [[ -z "${completed_pid}" || -z "${gate_index_by_pid[${completed_pid}]:-}" ]]; then
    echo "wait returned an unknown GitHub gate process" >&2
    exit 1
  fi
  index="${gate_index_by_pid[${completed_pid}]}"
  exit_codes[index]="${exit_code}"
  if [[ "${exit_code}" -ne 0 ]]; then
    overall_status=1
  fi
  printf 'completed gate: %s (exit %s)\n' \
    "${gate_names[${index}]}" \
    "${exit_code}"
  next_remaining=()
  for pid in "${remaining_pids[@]}"; do
    if [[ "${pid}" != "${completed_pid}" ]]; then
      next_remaining+=("${pid}")
    fi
  done
  remaining_pids=("${next_remaining[@]}")
done

for index in "${!gate_names[@]}"; do
  gate_name="${gate_names[${index}]}"
  gate_target="${gate_targets[${index}]}"
  gate_log="${artifact_dir}/${gate_name}.log"
  printf '%s\t%s\t%s\t%s\n' \
    "${gate_name}" \
    "${gate_target}" \
    "${exit_codes[${index}]}" \
    "${gate_log}" >>"${summary_file}"
done

column -t -s $'\t' "${summary_file}" 2>/dev/null || cat "${summary_file}"

if [[ "${overall_status}" -ne 0 ]]; then
  for index in "${!gate_names[@]}"; do
    gate_name="${gate_names[${index}]}"
    gate_log="${artifact_dir}/${gate_name}.log"
    if [[ "${exit_codes[${index}]}" -ne 0 ]]; then
      printf '\nfailed gate: %s\nlog: %s\n' "${gate_name}" "${gate_log}" >&2
      tail -n 40 "${gate_log}" >&2
    fi
  done
fi

exit "${overall_status}"
