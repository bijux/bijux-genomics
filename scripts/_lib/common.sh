#!/usr/bin/env sh
set -eu
LC_ALL=C
export LC_ALL

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 127
  }
}

require_env() {
  var_name="$1"
  eval "var_value=\${$var_name-}"
  [ -n "$var_value" ] || {
    echo "missing required env var: $var_name" >&2
    exit 2
  }
}

repo_root() {
  cd "$(dirname "$0")/.." && pwd
}

ensure_artifacts_dir() {
  dir_path="$1"
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
