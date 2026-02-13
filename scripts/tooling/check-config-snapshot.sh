#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BASELINE="$ROOT_DIR/configs/schema/config_tree.snapshot"
ACTUAL="${TEST_TMP_DIR:-$ROOT_DIR/artifacts/tmp}/config_tree.snapshot.actual"
mkdir -p "$(dirname "$ACTUAL")"

{
  echo '# schema_version = 1'
  echo '# owner = bijux-dna-infra'
  find "$ROOT_DIR/configs" -type f | sed "s#^$ROOT_DIR/##" | sort
} > "$ACTUAL"

if ! diff -u "$BASELINE" "$ACTUAL"; then
  echo "config snapshot drift detected; regenerate via scripts/tooling/generate-config-tree-snapshot.sh" >&2
  exit 1
fi

echo "config snapshot: OK"
