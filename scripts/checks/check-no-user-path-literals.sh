#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
banned='/(home|Users)/bijan/'

matches="$(rg -n --pcre2 "$banned" \
  crates scripts makefiles .github Makefile \
  --glob '!docs/**' \
  --glob '!examples/**' \
  --glob '!**/*.md' || true)"

if [[ -n "$matches" ]]; then
  echo "user-path-literal-check: FAILED"
  echo "$matches"
  exit 1
fi

echo "user-path-literal-check: OK"
