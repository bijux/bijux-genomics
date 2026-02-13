#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

lock="$ROOT_DIR/configs/ci/registry/tool_registry_lock.sha256"
lock_rules="$ROOT_DIR/configs/ci/registry/LOCK_RULES.md"

require_file "$lock_rules"
expected=$("$ROOT_DIR/scripts/domain/lock-registry.sh" --print)
actual=$(tr -d ' \t\n\r' < "$lock")

if [[ "$expected" != "$actual" ]]; then
  echo "tool-registry-lock: lock mismatch for configs/ci/registry/tool_registry_lock.sha256" >&2
  echo "tool-registry-lock: rules documented in configs/ci/registry/LOCK_RULES.md" >&2
  echo "tool-registry-lock: run ./scripts/run.sh domain lock-registry" >&2
  exit 1
fi

echo "tool-registry-lock: OK"
