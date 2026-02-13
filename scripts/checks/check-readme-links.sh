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
cd "$ROOT_DIR"

failed=0
for readme in scripts/README.md scripts/*/README.md scripts/*/*/README.md; do
  [[ -f "$readme" ]] || continue
  while IFS= read -r path; do
    [[ -z "$path" ]] && continue
    if [[ ! -e "$path" ]]; then
      echo "check-readme-links: missing path '$path' referenced from $readme" >&2
      failed=1
    fi
  done < <(rg -No '\`((scripts|configs|artifacts|containers|docs|domain|makefiles|crates)/[^` ]+)\`' "$readme" -r '$1' || true)
done

if [[ $failed -ne 0 ]]; then
  exit 1
fi

echo "check-readme-links: OK"
