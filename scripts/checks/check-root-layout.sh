#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

allowed=(assets bin configs containers crates docs domain examples makefiles scripts artifacts target)
offenders=()

for entry in */ ; do
  name="${entry%/}"
  [[ -d "${name}" ]] || continue
  if [[ "${name}" == .* ]]; then
    continue
  fi
  ok=0
  for allow in "${allowed[@]}"; do
    if [[ "${name}" == "${allow}" ]]; then
      ok=1
      break
    fi
  done
  if [[ "${ok}" -eq 0 ]]; then
    offenders+=("${name}")
  fi
done

if [[ "${#offenders[@]}" -gt 0 ]]; then
  printf 'root layout violations (unexpected top-level dirs):\n' >&2
  printf '  - %s\n' "${offenders[@]}" >&2
  printf 'move code into crates/, data into assets/, config into configs/, scripts into scripts/.\n' >&2
  exit 1
fi

echo "root-layout-check: OK"
