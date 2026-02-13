#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"

parse_network_allowed() {
  local spec_file="$1"
  local curr_path=""
  local curr_allowed="false"
  while IFS= read -r line; do
    case "$line" in
      "[[script]]")
        if [[ -n "$curr_path" ]]; then
          printf '%s\t%s\n' "$curr_path" "$curr_allowed"
        fi
        curr_path=""
        curr_allowed="false"
        ;;
      path\ =\ \"*\")
        curr_path="${line#path = \"}"
        curr_path="${curr_path%\"}"
        ;;
      network_allowed\ =\ true) curr_allowed="true" ;;
      network_allowed\ =\ false) curr_allowed="false" ;;
    esac
  done <"$spec_file"
  if [[ -n "$curr_path" ]]; then
    printf '%s\t%s\n' "$curr_path" "$curr_allowed"
  fi
}

viol=()
while IFS=$'\t' read -r rel net; do
  [[ -n "$rel" ]] || continue
  [[ -f "$ROOT_DIR/$rel" ]] || continue
  if rg -n '\bcurl\b|\bwget\b|git[[:space:]]+clone\b' "$ROOT_DIR/$rel" >/dev/null 2>&1; then
    if [[ "$net" != "true" ]]; then
      viol+=("$rel uses network command(s) but network_allowed != true")
    fi
  fi
done < <(parse_network_allowed "$SPEC")

# Also enforce for script files not listed in SUPPORTED (except experimental).
while IFS= read -r abs; do
  [[ -n "$abs" ]] || continue
  rel="${abs#$ROOT_DIR/}"
  [[ "$rel" == scripts/experimental/* ]] && continue
  if ! rg -n "^path = \"$rel\"$" "$SPEC" >/dev/null 2>&1; then
    if rg -n '\bcurl\b|\bwget\b|git[[:space:]]+clone\b' "$abs" >/dev/null 2>&1; then
      viol+=("$rel uses network command(s) but is not declared in scripts/SUPPORTED.toml with network_allowed=true")
    fi
  fi
done < <(find "$ROOT_DIR/scripts" -type f -name '*.sh' | sort)

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "check-network-usage: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "check-network-usage: OK"
