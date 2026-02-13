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

offenders=$(find "$ROOT/docs" -type f \( -iname '*.png' -o -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.gif' -o -iname '*.svg' -o -iname '*.webp' \) \
  | sed "s#^$ROOT/##" \
  | grep -v '^docs/assets/' || true)

if [[ -n "$offenders" ]]; then
  echo "doc-assets: images must live under docs/assets/" >&2
  printf '%s\n' "$offenders" >&2
  exit 1
fi

echo "doc-assets: OK"
