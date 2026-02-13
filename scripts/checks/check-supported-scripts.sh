#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"

parse_supported_toml() {
  local spec_file="$1"
  local curr_path=""
  while IFS= read -r line; do
    if [[ "$line" == path\ =\ \"*\" ]]; then
      curr_path="${line#path = \"}"
      curr_path="${curr_path%\"}"
    fi
    case "$line" in
      ci_allowed\ =\ true) [[ -n "$curr_path" ]] && printf '%s\ttrue\n' "$curr_path" ;;
      ci_allowed\ =\ false) [[ -n "$curr_path" ]] && printf '%s\tfalse\n' "$curr_path" ;;
    esac
  done < "$spec_file"
}

[[ -f "$SPEC" ]] || { echo "supported-scripts: missing spec: $SPEC" >&2; exit 1; }

listed_paths=$(parse_supported_toml "$SPEC" | cut -f1 | sort -u)
while IFS= read -r p; do
  [[ -n "$p" ]] || continue
  [[ -f "$ROOT_DIR/$p" ]] || { echo "supported-scripts: listed script file missing: $p" >&2; exit 1; }
done <<< "$listed_paths"

referenced=$(grep -RhoE 'scripts/[A-Za-z0-9_./-]+\.sh' "$ROOT_DIR/Makefile" "$ROOT_DIR/makefiles" | sort -u)
missing=()
while IFS= read -r p; do
  [[ -n "$p" ]] || continue
  if ! grep -qx "$p" <<< "$listed_paths"; then
    missing+=("$p")
  fi
done <<< "$referenced"

if [[ ${#missing[@]} -gt 0 ]]; then
  echo "supported-scripts: scripts referenced by make/CI but not listed in scripts/SUPPORTED.toml:" >&2
  printf '%s\n' "${missing[@]}" >&2
  exit 1
fi

echo "supported-scripts: OK"
