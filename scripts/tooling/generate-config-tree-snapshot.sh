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
OUT="$ROOT_DIR/configs/schema/config_tree.snapshot"
TMP="${TEST_TMP_DIR:-$ROOT_DIR/artifacts/tmp}/config_tree.snapshot.tmp"
MARKER_DIR="${ROOT_DIR}/artifacts/configs"
MARKER_FILE="${MARKER_DIR}/config_tree_snapshot.marker"
mkdir -p "$(dirname "$OUT")" "$(dirname "$TMP")"
mkdir -p "$MARKER_DIR"

{
  echo '# GENERATED - DO NOT EDIT'
  echo '# generator = scripts/tooling/generate-config-tree-snapshot.sh'
  echo '# schema_version = 1'
  echo '# owner = bijux-dna-infra'
  find "$ROOT_DIR/configs" -type f | sed "s#^$ROOT_DIR/##" | sort
} > "$TMP"

mv "$TMP" "$OUT"
sha=$(shasum -a 256 "$OUT" | awk '{print $1}')
{
  echo "generator=scripts/tooling/generate-config-tree-snapshot.sh"
  echo "snapshot_sha256=$sha"
} > "$MARKER_FILE"
echo "generated $OUT"
