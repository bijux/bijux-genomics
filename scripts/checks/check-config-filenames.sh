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
failed=0

while IFS= read -r file; do
  [[ -n "$file" ]] || continue
  rel="${file#$ROOT_DIR/}"
  case "$rel" in
    configs/OWNERS.toml|\
    configs/ci/registry/LOCK_RULES.md|\
    configs/docs/requirements.lock.txt|\
    configs/hpc/rsync/pull-full-excludes.txt|\
    configs/hpc/rsync/pull-results-includes.txt|\
    configs/hpc/rsync/push-excludes.txt|\
    configs/runtime/profiles/README.md|\
    configs/schema/CONFIG_SCHEMA_RULES.md|\
    configs/schema/validate.py|\
    configs/vcf/panels/locks/lock.json|\
    configs/vcf/panels/locks/lock.json.sha256)
      continue
      ;;
  esac
  base="$(basename "$file")"
  if [[ ! "$base" =~ ^[a-z0-9_]+\.(toml|ya?ml|md|snapshot|sha256|txt)$ ]]; then
    echo "config-filenames: non-snake_case name: $rel" >&2
    failed=1
  fi
done < <(find "$ROOT_DIR/configs" -type f | sort)

if [[ $failed -ne 0 ]]; then
  exit 1
fi

echo "config-filenames: OK"
