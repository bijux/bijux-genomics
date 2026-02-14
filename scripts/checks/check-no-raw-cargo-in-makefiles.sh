#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

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

if ! command -v rg >/dev/null 2>&1; then
  echo "raw-cargo-policy: ripgrep (rg) is required" >&2
  exit 127
fi

matches=$(rg -n "^\t@?.*\\bcargo([[:space:]]|$)" Makefile makefiles/*.mk || true)
if [[ -n "$matches" ]]; then
  violations=$(printf '%s\n' "$matches" | rg -v "(install once: cargo install|policy-no-raw-cargo)" || true)
  if [[ -n "$violations" ]]; then
    echo "raw-cargo-policy(makefiles): raw cargo mentions are forbidden; route through scripts/tooling/ci-*.sh or scripts/run.sh tooling ..." >&2
    printf '%s\n' "$violations" >&2
    exit 1
  fi
fi
echo "raw-cargo-policy(makefiles): OK (no raw cargo in makefiles)"

tool_matches=$(rg -n "(^|[^[:alnum:]_])(rustup|pip)([[:space:]]|$)|python[0-9.]*[[:space:]]+-m[[:space:]]+venv\\b" Makefile makefiles/*.mk || true)
if [[ -n "$tool_matches" ]]; then
  tool_violations=$(printf '%s\n' "$tool_matches" | rg -v "scripts/tooling/" || true)
  if [[ -n "$tool_violations" ]]; then
    echo "raw-tooling-policy(makefiles): direct rustup/pip/python -m venv found; route via scripts/tooling/*.sh" >&2
    printf '%s\n' "$tool_violations" >&2
    exit 1
  fi
fi
echo "raw-tooling-policy(makefiles): OK"

direct_script_calls=$(rg -n "^\t@?.*scripts/[A-Za-z0-9_./-]+\\.sh" Makefile makefiles/*.mk || true)
if [[ -n "$direct_script_calls" ]]; then
  bad_direct=$(printf '%s\n' "$direct_script_calls" | rg -v "scripts/run\\.sh" || true)
  if [[ -n "$bad_direct" ]]; then
    echo "makefile-script-surface: makefiles must invoke scripts through scripts/run.sh (or approved thin wrappers)" >&2
    printf '%s\n' "$bad_direct" >&2
    exit 1
  fi
fi
echo "makefile-script-surface: OK"

ci_allowed_paths=$(parse_supported_toml "$ROOT_DIR/scripts/SUPPORTED.toml" | awk -F'\t' '$2=="true"{print $1}' | sort -u)

viol=()
while IFS= read -r line; do
  [[ -n "$line" ]] || continue
  # scripts/run.sh <group> <command> -> scripts/<group>/make.sh
  if [[ "$line" =~ scripts/run\.sh[[:space:]]+([a-z_]+)[[:space:]]+([A-Za-z0-9_./-]+) ]]; then
    group="${BASH_REMATCH[1]}"
    path="scripts/${group}/make.sh"
    if ! grep -qx "$path" <<< "$ci_allowed_paths"; then
      viol+=("$line -> $path not ci_allowed=true")
    fi
  fi
  while [[ "$line" =~ (scripts/[A-Za-z0-9_./-]+\.sh) ]]; do
    path="${BASH_REMATCH[1]}"
    if [[ "$path" != "scripts/run.sh" ]] && ! grep -qx "$path" <<< "$ci_allowed_paths"; then
      viol+=("$line -> $path not ci_allowed=true")
    fi
    line="${line#*${path}}"
  done
done < <(cat Makefile makefiles/*.mk)

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "ci-allowed-policy(makefiles): make/CI may call only scripts with ci_allowed=true" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "ci-allowed-policy(makefiles): OK"
