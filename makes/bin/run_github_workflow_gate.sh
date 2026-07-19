#!/usr/bin/env bash
set -euo pipefail

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "required tool is unavailable: $1" >&2
    exit 1
  fi
}

require_tool git
require_tool sed

repo_root="$(git rev-parse --show-toplevel)"
artifact_dir="${GITHUB_WORKFLOW_ARTIFACT_DIR:-${repo_root}/artifacts/github-all}"
make_bin="${GITHUB_WORKFLOW_MAKE_BIN:-make}"
summary_file="${artifact_dir}/summary.tsv"
workflow_file="${repo_root}/.github/workflows/ci.yml"

require_tool "${make_bin}"

if [[ ! -f "${workflow_file}" ]]; then
  echo "GitHub CI workflow is unavailable: ${workflow_file}" >&2
  exit 1
fi

gate_names=()
gate_targets=()
while IFS= read -r workflow_target; do
  gate_target="${workflow_target}"
  if [[ "${gate_target}" == "test" ]]; then
    gate_target="test-all"
  fi
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
rm -f "${artifact_dir}"/*.exit.status "${artifact_dir}"/*.log "${summary_file}"
printf 'gate\ttarget\texit_code\tlog\n' >"${summary_file}"

pids=()

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
  gate_status="${artifact_dir}/${gate_name}.exit.status"
  (
    set +e
    "${make_bin}" --no-print-directory "${gate_target}" >"${gate_log}" 2>&1
    exit_code=$?
    set -e
    printf '%s\n' "${exit_code}" >"${gate_status}"
    exit "${exit_code}"
  ) &
  pids+=("$!")
done

overall_status=0
for index in "${!gate_names[@]}"; do
  gate_name="${gate_names[${index}]}"
  gate_target="${gate_targets[${index}]}"
  gate_log="${artifact_dir}/${gate_name}.log"
  if wait "${pids[${index}]}"; then
    exit_code=0
  else
    exit_code=$?
    overall_status=1
  fi
  printf '%s\t%s\t%s\t%s\n' \
    "${gate_name}" \
    "${gate_target}" \
    "${exit_code}" \
    "${gate_log}" >>"${summary_file}"
done

column -t -s $'\t' "${summary_file}" 2>/dev/null || cat "${summary_file}"

if [[ "${overall_status}" -ne 0 ]]; then
  for index in "${!gate_names[@]}"; do
    gate_name="${gate_names[${index}]}"
    gate_status="${artifact_dir}/${gate_name}.exit.status"
    gate_log="${artifact_dir}/${gate_name}.log"
    if [[ "$(cat "${gate_status}")" -ne 0 ]]; then
      printf '\nfailed gate: %s\nlog: %s\n' "${gate_name}" "${gate_log}" >&2
      tail -n 40 "${gate_log}" >&2
    fi
  done
fi

exit "${overall_status}"
