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

status=0

while IFS= read -r dataset_dir; do
  checksum_file="${dataset_dir}/CHECKSUMS.sha256"
  if [[ ! -f "${checksum_file}" ]]; then
    echo "assets-drift: missing checksum file: ${checksum_file#"$ROOT/"}" >&2
    status=1
    continue
  fi

  (
    cd "$dataset_dir"
    shasum -a 256 -c CHECKSUMS.sha256 >/dev/null
  ) || {
    echo "assets-drift: checksum mismatch in ${dataset_dir#"$ROOT/"}" >&2
    status=1
  }
done < <(find "${ROOT}/assets/toy" -mindepth 1 -maxdepth 1 -type d | sort)

while IFS= read -r pub_dir; do
  manifest="${pub_dir}/MANIFEST.toml"
  if [[ ! -f "${manifest}" ]]; then
    echo "assets-drift: missing publication manifest: ${manifest#"$ROOT/"}" >&2
    status=1
  fi
done < <(find "${ROOT}/assets/publications" -mindepth 1 -maxdepth 1 -type d | sort)

untracked_assets="$(git -C "$ROOT" ls-files --others --exclude-standard -- assets)"
if [[ -n "${untracked_assets}" ]]; then
  echo "assets-drift: untracked files under assets/ are not allowed:" >&2
  printf '%s\n' "${untracked_assets}" >&2
  status=1
fi

if [[ "${status}" -ne 0 ]]; then
  exit "${status}"
fi

echo "assets-drift: OK"
