#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
tracked="$(git ls-files artifacts || true)"
if [[ -n "${tracked}" ]]; then
  echo "tracked files under artifacts/ are forbidden:" >&2
  echo "${tracked}" >&2
  exit 1
fi

echo "artifacts-tracked-check: OK"
