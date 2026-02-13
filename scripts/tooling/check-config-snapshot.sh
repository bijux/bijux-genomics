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
MARKER_FILE="$ROOT_DIR/artifacts/configs/config_tree_snapshot.marker"
mkdir -p "$(dirname "$ACTUAL")"

{
  echo '# GENERATED - DO NOT EDIT'
  echo '# generator = scripts/tooling/generate-config-tree-snapshot.sh'
  echo '# schema_version = 1'
  echo '# owner = bijux-dna-infra'
  find "$ROOT_DIR/configs" -type f | sed "s#^$ROOT_DIR/##" | sort
} > "$ACTUAL"

if ! diff -u "$BASELINE" "$ACTUAL"; then
  echo "config snapshot drift detected; regenerate via scripts/tooling/generate-config-tree-snapshot.sh" >&2
  exit 1
fi

if ! rg -q '^# GENERATED - DO NOT EDIT$' "$BASELINE"; then
  echo "config snapshot header missing GENERATED marker" >&2
  exit 1
fi
if ! rg -q '^# generator = scripts/tooling/generate-config-tree-snapshot.sh$' "$BASELINE"; then
  echo "config snapshot header missing generator contract" >&2
  exit 1
fi

if [[ ! -f "$MARKER_FILE" ]]; then
  echo "config snapshot marker missing: run scripts/tooling/generate-config-tree-snapshot.sh" >&2
  exit 1
fi
marker_sha=$(awk -F= '$1=="snapshot_sha256"{print $2}' "$MARKER_FILE" | tr -d ' \t\r\n')
actual_sha=$(shasum -a 256 "$BASELINE" | awk '{print $1}')
if [[ -z "$marker_sha" || "$marker_sha" != "$actual_sha" ]]; then
  echo "config snapshot marker is stale: run scripts/tooling/generate-config-tree-snapshot.sh" >&2
  exit 1
fi

echo "config snapshot: OK"
