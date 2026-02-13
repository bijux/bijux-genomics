#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
_COMMON_LIB_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "${_COMMON_LIB_DIR}/runtime.sh"

_print_common_help() {
  local caller="${BASH_SOURCE[2]:-${BASH_SOURCE[1]:-$0}}"
  local rel="${caller##*/scripts/}"
  if [[ "$rel" == "$caller" ]]; then
    rel="$caller"
  else
    rel="scripts/$rel"
  fi
  cat <<EOF
Usage: $caller [--help] [args...]

Script contract:
- Requires: declared in ${rel%/*}/README.md
- Exit codes: declared in ${rel%/*}/README.md
EOF
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  _print_common_help
  exit 0
fi

require_stable_env() {
  export TZ="${TZ:-UTC}"
  export LC_ALL="${LC_ALL:-C}"
  [[ "$TZ" == "UTC" ]] || {
    echo "unstable TZ: expected UTC, got $TZ" >&2
    exit 2
  }
  [[ "$LC_ALL" == "C" ]] || {
    echo "unstable LC_ALL: expected C, got $LC_ALL" >&2
    exit 2
  }
}

require_env() {
  local var_name="$1"
  local var_value="${!var_name-}"
  [[ -n "$var_value" ]] || {
    echo "missing required env var: $var_name" >&2
    exit 2
  }
}

repo_root() {
  cd "$(dirname "${BASH_SOURCE[1]:-$0}")/.." && pwd
}

ensure_artifacts_dir() {
  local dir_path="$1"
  case "$dir_path" in
    artifacts/*|*/artifacts/*|"${ISO_ROOT:-}"/*)
      mkdir -p "$dir_path"
      ;;
    *)
      echo "refusing to write outside artifacts/ or ISO_ROOT: $dir_path" >&2
      exit 2
      ;;
  esac
}

compat_sed_inplace() {
  local expr="$1"
  local file="$2"
  sed -i.bak "$expr" "$file" 2>/dev/null || sed -i '' "$expr" "$file"
  rm -f "${file}.bak"
}

compat_readlink_f() {
  local target="$1"
  python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$target"
}
