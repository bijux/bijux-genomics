#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

viol=()
while IFS= read -r line; do
  [[ -n "$line" ]] || continue
  if [[ "$line" == *"scripts/"*".sh"* ]]; then
    if [[ "$line" != *"./scripts/run.sh "* && "$line" != *"scripts/run.sh "* ]]; then
      # ignore docs/examples references in markdown or comments.
      if [[ "$line" != \#* ]]; then
        viol+=("$line")
      fi
    fi
  fi
done < <(grep -RhoE '^\s*[^#].*scripts/[A-Za-z0-9_./-]+\.sh([^A-Za-z0-9_./-]|$).*' "$ROOT_DIR/Makefile" "$ROOT_DIR/makefiles" | sort -u)

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "check-script-entrypoint: makefiles must call scripts via scripts/run.sh only" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "check-script-entrypoint: OK"
