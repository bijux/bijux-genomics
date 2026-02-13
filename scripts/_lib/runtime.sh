#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

die() {
  local msg="${1:-fatal error}"
  printf 'error: %s\n' "$msg" >&2
  exit "${2:-1}"
}

warn() {
  local msg="${1:-warning}"
  printf 'warn: %s\n' "$msg" >&2
}

info() {
  local msg="${1:-}"
  printf 'info: %s\n' "$msg" >&2
}

require_cmd() {
  local cmd="${1:-}"
  [[ -n "$cmd" ]] || die "require_cmd expects a command name" 2
  command -v "$cmd" >/dev/null 2>&1 || die "missing required command: $cmd" 127
}

require_file() {
  local p="${1:-}"
  [[ -f "$p" ]] || die "missing required file: $p" 2
}

require_dir() {
  local p="${1:-}"
  [[ -d "$p" ]] || die "missing required directory: $p" 2
}

write_artifact() {
  local path="${1:-}"
  shift || true
  [[ "$path" == artifacts/* || "$path" == */artifacts/* || ( -n "${ISO_ROOT:-}" && "$path" == "$ISO_ROOT"/* ) ]] || {
    die "refusing to write outside artifacts/ or ISO_ROOT: $path" 2
  }
  mkdir -p "$(dirname "$path")"
  if [[ $# -gt 0 ]]; then
    printf '%s\n' "$*" >"$path"
  else
    cat >"$path"
  fi
}

parse_common_flags() {
  # Consumes leading common flags and leaves remaining args on "$@".
  # Sets DRY_RUN=1 and/or VERBOSE=1 when requested.
  while [[ $# -gt 0 ]]; do
    case "${1:-}" in
      --dry-run)
        export DRY_RUN=1
        shift
        ;;
      --verbose)
        export VERBOSE=1
        shift
        ;;
      --help|-h)
        # caller handles help output; stop parsing to keep behavior explicit.
        break
        ;;
      *)
        break
        ;;
    esac
  done
  printf '%s\0' "$@"
}
