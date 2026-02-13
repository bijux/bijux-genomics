#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

registry="$ROOT_DIR/configs/ci/registry/tool_registry.toml"
lock="$ROOT_DIR/configs/ci/registry/tool_registry_lock.sha256"

require_cmd shasum
new_sha=$(shasum -a 256 "$registry" | awk '{print $1}')
printf '%s\n' "$new_sha" > "$lock"

echo "updated $lock"
