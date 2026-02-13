#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
MANIFEST_DIR="${MANIFEST_DIR:-$ROOT_DIR/artifacts/container}"

if [ ! -d "$MANIFEST_DIR" ]; then
  echo "no manifests found: $MANIFEST_DIR" >&2
  exit 2
fi

printf "tool\truntime\tresult\tlog\n"
for f in "$MANIFEST_DIR"/*.json; do
  [ -e "$f" ] || continue
  tool=$(awk -F'"' '/"tool"/ {print $4; exit}' "$f")
  runtime=$(awk -F'"' '/"runtime"/ {print $4; exit}' "$f")
  status=$(awk -F'"' '/"status"/ {print $4; exit}' "$f")
  log="$MANIFEST_DIR/logs/${runtime}/${tool}.log"
  printf "%s\t%s\t%s\t%s\n" "$tool" "$runtime" "$status" "$log"
done | sort
