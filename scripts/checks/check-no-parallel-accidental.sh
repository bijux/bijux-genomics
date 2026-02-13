#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'USAGE'
Usage: scripts/checks/check-no-parallel-accidental.sh
USAGE
}

if [[ "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"
viol=()
while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  f="$ROOT_DIR/$rel"
  [[ -f "$f" ]] || continue
  while IFS= read -r p; do
    [[ -n "$p" ]] || continue
    case "$p" in
      artifacts/isolates/*|artifacts/tmp/*|artifacts/inventory/*|artifacts/test-logs/*|artifacts/coverage/*|artifacts/docs/*|artifacts/policies/*|artifacts/assets-refresh/*)
        continue
        ;;
    esac
    # fixed output roots require run-id style variableization to avoid accidental parallel collisions.
    if rg -n "${p//\//\/}" "$f" | rg -vq '\$\{?(ISO_ROOT|RUN_ID|TAG|TIMESTAMP|UUID|RANDOM|tmp|TMP)' ; then
      viol+=("$rel -> fixed artifact path may collide in parallel runs: $p")
    fi
  done < <(rg -n -o 'artifacts/[A-Za-z0-9._/-]+' "$f" | cut -d: -f3 | sort -u)
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "check-no-parallel-accidental: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "check-no-parallel-accidental: OK"
