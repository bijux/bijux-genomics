#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

offenders=()

for path in "$ROOT_DIR"/coverage "$ROOT_DIR"/target-* "$ROOT_DIR"/target-docs; do
  [ -e "$path" ] || continue
  base=$(basename "$path")
  case "$base" in
    target-docs) offenders+=("$base") ;;
    coverage) offenders+=("$base") ;;
    target-*) offenders+=("$base") ;;
  esac
done

if [ "${#offenders[@]}" -ne 0 ]; then
  {
    echo "root-pollution: forbidden repo-root outputs detected"
    for item in "${offenders[@]}"; do
      echo " - $item"
    done
    echo "Use isolate-scoped ISO_ROOT outputs or artifacts/container/* outputs instead."
  } >&2
  exit 1
fi

echo "root-pollution: OK"
