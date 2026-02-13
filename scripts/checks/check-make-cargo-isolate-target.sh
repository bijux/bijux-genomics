#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

viol=()

# Cargo is generally forbidden in makefiles, but if it appears it must be isolate-scoped.
while IFS= read -r line; do
  [[ -n "$line" ]] || continue
  # shell recipe line in cargo.mk that invokes cargo directly
  if [[ "$line" =~ ^[[:space:]]*@?.*\bcargo([[:space:]]|$) ]]; then
    if [[ "$line" != *"CARGO_TARGET_DIR"* ]] || [[ "$line" != *"ISO_ROOT"* ]]; then
      viol+=("makefiles/cargo.mk:${line}")
    fi
  fi
done < "$ROOT_DIR/makefiles/cargo.mk"

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "make-cargo-isolate-target: direct cargo in cargo.mk must set CARGO_TARGET_DIR under ISO_ROOT" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "make-cargo-isolate-target: OK"
