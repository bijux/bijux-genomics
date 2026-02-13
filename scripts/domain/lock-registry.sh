#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

LOCK_DOC="$ROOT_DIR/configs/ci/registry/LOCK_RULES.md"
LOCK_FILE="$ROOT_DIR/configs/ci/registry/tool_registry_lock.sha256"
MARKER_FILE="$ROOT_DIR/artifacts/configs/tool_registry_lock.marker"

print_only=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --print) print_only=1 ;;
    --help|-h)
      cat <<EOF
Usage: $0 [--print]
Computes lock per configs/ci/registry/LOCK_RULES.md.
EOF
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 2
      ;;
  esac
  shift
done

require_cmd shasum
require_file "$LOCK_DOC"

inputs=(
  "configs/ci/registry/tool_registry.toml"
  "configs/ci/registry/tool_registry_experimental.toml"
  "configs/ci/registry/tool_registry_vcf.toml"
  "configs/ci/registry/tool_registry_vcf_downstream.toml"
  "configs/ci/registry/domains.toml"
  "configs/ci/registry/deprecations.toml"
)

payload=""
for rel in "${inputs[@]}"; do
  abs="$ROOT_DIR/$rel"
  require_file "$abs"
  file_sha=$(shasum -a 256 "$abs" | awk '{print $1}')
  payload+="${rel} ${file_sha}"$'\n'
done

lock_sha=$(printf '%s' "$payload" | shasum -a 256 | awk '{print $1}')
if [[ "$print_only" == "1" ]]; then
  printf '%s\n' "$lock_sha"
  exit 0
fi

printf '%s\n' "$lock_sha" > "$LOCK_FILE"
mkdir -p "$(dirname "$MARKER_FILE")"
cat > "$MARKER_FILE" <<EOF
generated_by=scripts/domain/lock-registry.sh
lock_sha256=$lock_sha
EOF
echo "updated $LOCK_FILE (rules: configs/ci/registry/LOCK_RULES.md)"
