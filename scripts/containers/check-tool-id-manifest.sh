#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

manifest="$ROOT_DIR/containers/TOOL_IDS.txt"
if [[ ! -f "$manifest" ]]; then
  echo "missing tool id manifest: containers/TOOL_IDS.txt" >&2
  exit 1
fi

TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
tmp_expected="$(mktemp "$TMP_ROOT/tool-ids-expected.XXXXXX")"
tmp_files="$(mktemp "$TMP_ROOT/tool-ids-files.XXXXXX")"
trap 'rm -f "$tmp_expected" "$tmp_files"' EXIT INT TERM

"$SCRIPT_DIR/generate-tool-ids.sh" "$tmp_expected" >/dev/null
if ! diff -u "$manifest" "$tmp_expected" >/dev/null; then
  echo "containers/TOOL_IDS.txt drift; regenerate with scripts/containers/generate-tool-ids.sh" >&2
  exit 1
fi

grep -vE '^(#|$)' "$manifest" | awk -F'\t' '{print $1}' | sort -u > "$tmp_expected"

{
  find "$ROOT_DIR/containers/docker/arm64" -type f -name 'Dockerfile.*' -print \
    | sed -E 's#^.*/Dockerfile\.##'
  find "$ROOT_DIR/containers/apptainer/bijux" -type f -name '*.def' -print \
    | sed -E 's#^.*/##; s#\.def$##'
  find "$ROOT_DIR/containers/apptainer/non-bijux" -type f -name '*.def' -print \
    | sed -E 's#^.*/##; s#\.def$##'
} | sort -u > "$tmp_files"

unknown="$(comm -23 "$tmp_files" "$tmp_expected" || true)"
if [[ -n "$unknown" ]]; then
  echo "container filename tool IDs missing from containers/TOOL_IDS.txt:" >&2
  printf '%s\n' "$unknown" >&2
  exit 1
fi

echo "tool id manifest: OK"
