#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
LOG_PATH="${1:-artifacts/test-logs/latest.log}"

if [[ ! -f "${LOG_PATH}" ]]; then
  echo "missing log file: ${LOG_PATH}" >&2
  echo "hint: run make test | tee artifacts/test-logs/<name>.log and symlink/copy to artifacts/test-logs/latest.log" >&2
  exit 0
fi

echo "triage source: ${LOG_PATH}"
echo

failures="$(rg -No '([A-Za-z0-9_:-]+::)+[A-Za-z0-9_:-]+' "${LOG_PATH}" | sort -u || true)"
if [[ -z "${failures}" ]]; then
  echo "no test-like failure identifiers found"
  exit 0
fi

bucket() {
  local test_name="$1"
  case "${test_name}" in
    *guardrail*|*guardrails*|*policy_test_names_are_consistent*|*workspace_lints*)
      echo "guardrails"
      ;;
    *snapshot*|*insta*)
      echo "snapshots"
      ;;
    *registry*|*binding*|*supported_stages_and_tools_are_complete*)
      echo "ssot-registry"
      ;;
    *apptainer*|*smoke*|*environment*ensure*|*containers*)
      echo "apptainer-policy"
      ;;
    *spawn*|*process*|*command_new*)
      echo "spawn-policy"
      ;;
    *)
      echo "other"
      ;;
  esac
}

declare -A counts=()
declare -A items=()
while IFS= read -r test_name; do
  b="$(bucket "${test_name}")"
  counts["${b}"]=$(( ${counts["${b}"]:-0} + 1 ))
  items["${b}"]+=$'\n'"- ${test_name}"
done <<< "${failures}"

for name in guardrails snapshots ssot-registry apptainer-policy spawn-policy other; do
  count="${counts["${name}"]:-0}"
  if [[ "${count}" -gt 0 ]]; then
    echo "[${name}] ${count}"
    echo "${items["${name}"]}" | sed '/^$/d'
    echo
  fi
done
