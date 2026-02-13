#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ -z "${CI:-}" ]]; then
  echo "lock change discipline: SKIP (CI-only gate)"
  exit 0
fi

if ! git -C "$ROOT_DIR" rev-parse --verify HEAD^ >/dev/null 2>&1; then
  echo "lock change discipline: SKIP (no previous commit)"
  exit 0
fi

changed="$(git -C "$ROOT_DIR" diff --name-only HEAD^..HEAD -- containers/versions/versions.toml containers/versions/lock.json)"
has_versions=0
has_lock=0
if printf '%s\n' "$changed" | rg -qx 'containers/versions/versions.toml'; then
  has_versions=1
fi
if printf '%s\n' "$changed" | rg -qx 'containers/versions/lock.json'; then
  has_lock=1
fi

if [[ "$has_versions" -eq 1 && "$has_lock" -eq 0 ]]; then
  echo "lock change discipline: versions.toml changed but lock.json did not" >&2
  exit 1
fi
if [[ "$has_versions" -eq 0 && "$has_lock" -eq 1 ]]; then
  echo "lock change discipline: lock.json changed without versions.toml change" >&2
  exit 1
fi

echo "lock change discipline: OK"
