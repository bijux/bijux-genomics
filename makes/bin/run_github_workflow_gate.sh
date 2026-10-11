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
require_tool sed
require_tool cargo
require_tool cargo-nextest

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
workflow_file="${repo_root}/.github/workflows/ci.yml"
nextest_config_file="${NEXTEST_CONFIG_FILE:-${repo_root}/configs/rust/nextest.toml}"
nextest_profile="${NEXTEST_PROFILE_FAST:-fast-unit}"

require_tool "${make_bin}"

if [[ ! -f "${workflow_file}" ]]; then
  echo "GitHub CI workflow is unavailable: ${workflow_file}" >&2
  exit 1
fi

gate_names=()
gate_targets=()
while IFS= read -r workflow_target; do
  gate_target="${workflow_target}"
  already_registered=0
  for registered_target in "${gate_targets[@]:-}"; do
    if [[ "${registered_target}" == "${gate_target}" ]]; then
      already_registered=1
      break
    fi
  done
  if [[ "${already_registered}" -eq 1 ]]; then
    continue
  fi
  gate_name="${gate_target#_}"
  gate_name="${gate_name//_/-}"
  gate_names+=("${gate_name}")
  gate_targets+=("${gate_target}")
done < <(
  sed -n -E \
    -e 's/^[[:space:]]*run: "make ([A-Za-z0-9_-]+)"[[:space:]]*$/\1/p' \
    -e 's/^[[:space:]]*make ([A-Za-z0-9_-]+)[[:space:]]*$/\1/p' \
    "${workflow_file}"
)

if [[ "${#gate_targets[@]}" -eq 0 ]]; then
  echo "GitHub CI workflow declares no Make gates: ${workflow_file}" >&2
  exit 1
fi

mkdir -p "${artifact_dir}"
rm -f \
  "${artifact_dir}"/*.log \
  "${summary_file}"
printf 'gate\ttarget\texit_code\tlog\n' >"${summary_file}"

workspace_compile_log="${artifact_dir}/workspace-compile.log"
printf 'preparing shared workspace binaries\nlog: %s\n' "${workspace_compile_log}"
if (
  "${make_bin}" --no-print-directory _dev-dna-bin
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
  printf 'started gate: %s (make %s)\nlog: %s\n' \
    "${gate_name}" \
    "${gate_target}" \
    "${gate_log}"
  (
    "${make_bin}" --no-print-directory "${gate_target}" >"${gate_log}" 2>&1
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
