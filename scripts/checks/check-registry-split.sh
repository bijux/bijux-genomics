#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

stable="$ROOT_DIR/configs/ci/registry/tool_registry.toml"
experimental="$ROOT_DIR/configs/ci/registry/tool_registry_experimental.toml"

tmp_root="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$tmp_root"
mkdir -p "$tmp_root"
stable_ids=$(mktemp "$tmp_root/tmp-registry-stable.XXXXXX")
experimental_ids=$(mktemp "$tmp_root/tmp-registry-experimental.XXXXXX")
trap 'rm -f "$stable_ids" "$experimental_ids"' EXIT

rg '^id\s*=\s*"' "$stable" | sed -E 's/^id\s*=\s*"([^"]+)".*/\1/' | sort -u > "$stable_ids"
rg '^id\s*=\s*"' "$experimental" | sed -E 's/^id\s*=\s*"([^"]+)".*/\1/' | sort -u > "$experimental_ids"

both=$(comm -12 "$stable_ids" "$experimental_ids" || true)
if [[ -n "$both" ]]; then
  echo "registry-split: tool id appears in both stable and experimental registries:" >&2
  printf '%s\n' "$both" >&2
  exit 1
fi

echo "registry-split: OK"
