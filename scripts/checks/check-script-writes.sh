#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
ALLOWLIST="$ROOT_DIR/scripts/checks/supported_scripts.txt"

failed=0
while IFS= read -r rel; do
  [[ -z "$rel" ]] && continue
  file="$ROOT_DIR/$rel"
  [[ -f "$file" ]] || continue

  # Static guard: ban obvious absolute writes in supported scripts.
  if rg -n '(>|>>|cp |mv |rm -rf|mkdir -p)\s*/(tmp|var|opt|usr|etc|home|Users)\b' "$file" >/dev/null 2>&1; then
    echo "check-script-writes: forbidden absolute write path pattern in $rel" >&2
    failed=1
  fi

done < <(sed '/^\s*$/d' "$ALLOWLIST")

if [[ $failed -ne 0 ]]; then
  exit 1
fi

echo "check-script-writes: OK"
