#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

lock="$ROOT_DIR/configs/ci/registry/tool_registry_lock.sha256"
lock_rules="$ROOT_DIR/configs/ci/registry/LOCK_RULES.md"
marker="$ROOT_DIR/artifacts/configs/tool_registry_lock.marker"

require_file "$lock_rules"
expected=$("$ROOT_DIR/scripts/domain/lock-registry.sh" --print)
actual=$(tr -d ' \t\n\r' < "$lock")

if [[ "$expected" != "$actual" ]]; then
  echo "tool-registry-lock: lock mismatch for configs/ci/registry/tool_registry_lock.sha256" >&2
  echo "tool-registry-lock: rules documented in configs/ci/registry/LOCK_RULES.md" >&2
  echo "tool-registry-lock: run ./scripts/run.sh domain lock-registry" >&2
  exit 1
fi

if [[ ! -f "$marker" ]]; then
  echo "tool-registry-lock: missing marker $marker; run ./scripts/run.sh domain lock-registry" >&2
  exit 1
fi
marker_sha=$(awk -F= '$1=="lock_sha256"{print $2}' "$marker" | tr -d ' \t\r\n')
marker_gen=$(awk -F= '$1=="generated_by"{print $2}' "$marker" | tr -d ' \t\r\n')
if [[ "$marker_gen" != "scripts/domain/lock-registry.sh" || "$marker_sha" != "$actual" ]]; then
  echo "tool-registry-lock: marker stale or invalid; regenerate via ./scripts/run.sh domain lock-registry" >&2
  exit 1
fi

echo "tool-registry-lock: OK"
