#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

target="$ROOT_DIR/docs/30-operations/CONTAINER_LICENSE_INDEX.md"

"$SCRIPT_DIR/generate-license-metadata.sh" >/dev/null
git -C "$ROOT_DIR" diff --exit-code -- "$target" >/dev/null 2>&1 || {
  echo "license index drift: regenerate with scripts/containers/generate-license-metadata.sh" >&2
  exit 1
}
echo "license index generated: OK"
