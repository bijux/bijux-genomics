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
  if [[ -n "${remaining_pids[*]:-}" ]]; then
    # Unexpected supervisor errors must not leave target writers running.
    terminate_gates || status=1
    if [[ "${status}" -eq 0 ]]; then
      status=1
    fi
  fi
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
remaining_pids=()
exit_codes=()
launching_gate_index=""
declare -A gate_index_by_pid=()

publish_gate_result() {
  local index="$1"
  local status="$2"
  exit_codes[index]="${status}"
  printf '%s\t%s\t%s\t%s\n' \
    "${gate_names[index]}" "${gate_targets[index]}" "${status}" \
    "${artifact_dir}/${gate_names[index]}.log" >>"${summary_file}"
}

# Each asynchronous job is a separate process group, including its make/Cargo
# descendants. The foreground supervisor never signals its own process group.
set -m

# shellcheck disable=SC2329  # Called by the signal and EXIT handlers.
terminate_gates() {
  local pid status attempt
  local groups=("${remaining_pids[@]}")
  # A signal can arrive between starting a job and recording $!.
  while IFS= read -r pid; do
    if [[ -n "${pid}" && " ${groups[*]:-} " != *" ${pid} "* ]]; then
      groups+=("${pid}")
      if [[ -n "${launching_gate_index}" ]]; then
        gate_index_by_pid["${pid}"]="${launching_gate_index}"
      fi
    fi
  done < <(jobs -pr)
  for pid in "${groups[@]}"; do
    kill -TERM -- "-${pid}" 2>/dev/null || true
  done
  for ((attempt = 0; attempt < 100; attempt++)); do
    local running=0
    for pid in "${groups[@]}"; do
      if kill -0 -- "-${pid}" 2>/dev/null; then
        running=1
      fi
    done
    if [[ "${running}" -eq 0 ]]; then
      break
    fi
    sleep 0.05
  done
  for pid in "${groups[@]}"; do
    if kill -0 -- "-${pid}" 2>/dev/null; then
      kill -KILL -- "-${pid}" 2>/dev/null || true
    fi
  done
  for pid in "${groups[@]}"; do
    if wait "${pid}"; then
      status=0
    else
      status=$?
    fi
    if [[ -n "${gate_index_by_pid[${pid}]:-}" ]]; then
      publish_gate_result "${gate_index_by_pid[${pid}]}" "${status}"
    else
      printf 'workspace-preparation\t_dev-dna-bin/nextest-list\t%s\t%s\n' \
        "${status}" "${workspace_compile_log}" >>"${summary_file}"
    fi
  done
  remaining_pids=()
  for ((attempt = 0; attempt < 100; attempt++)); do
    local running=0
    for pid in "${groups[@]}"; do
      if kill -0 -- "-${pid}" 2>/dev/null; then
        running=1
      fi
    done
    if [[ "${running}" -eq 0 ]]; then
      return 0
    fi
    sleep 0.05
  done
  echo "GitHub gate process groups did not drain" >&2
  return 1
}

# shellcheck disable=SC2329  # Invoked indirectly by the signal traps.
interrupt_gates() {
  local status="$1"
  trap '' INT TERM
  terminate_gates
  exit "${status}"
}
trap 'interrupt_gates 130' INT
trap 'interrupt_gates 143' TERM

if [[ "${preparation_mode}" == "test-preparation" ]]; then
  require_tool cargo-nextest
  printf 'preparing shared workspace binaries\nlog: %s\n' "${workspace_compile_log}"
  (
    "${make_bin}" --no-print-directory _dev-dna-bin || exit $?
    cargo nextest list \
      --workspace \
      --all-features \
      --locked \
      --config-file "${nextest_config_file}" \
      --profile "${nextest_profile}" \
      --target-dir "${cargo_target_dir}" \
      >/dev/null
  ) >"${workspace_compile_log}" 2>&1 &
  preparation_pid="$!"
  remaining_pids=("${preparation_pid}")
  if wait "${preparation_pid}"; then
    remaining_pids=()
    printf 'prepared shared workspace binaries\n'
  else
    compile_status=$?
    remaining_pids=()
    printf 'workspace binary preparation failed (exit %s)\n' "${compile_status}" >&2
    tail -n 80 "${workspace_compile_log}" >&2
    exit "${compile_status}"
  fi
fi

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
  launching_gate_index="${index}"
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
  remaining_pids+=("${gate_pid}")
  gate_index_by_pid["${gate_pid}"]="${index}"
  launching_gate_index=""
done

overall_status=0
while [[ "${#remaining_pids[@]}" -gt 0 ]]; do
  completed_pid=""
  set +e
  wait -n -p completed_pid "${remaining_pids[@]}"
  exit_code=$?
  set -e
  if [[ -z "${completed_pid:-}" || -z "${gate_index_by_pid[${completed_pid}]:-}" ]]; then
    echo "wait returned an unknown GitHub gate process" >&2
    exit 1
  fi
  index="${gate_index_by_pid[${completed_pid}]}"
  publish_gate_result "${index}" "${exit_code}"
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
