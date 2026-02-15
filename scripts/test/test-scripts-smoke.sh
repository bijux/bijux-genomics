#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'USAGE'
Usage: scripts/test/test-scripts-smoke.sh [--help] [--dry-run]
Env:
  TEST_SCRIPTS_SMOKE_PROBE_DRY_RUN=1   enable --dry-run probes (off by default)
USAGE
}

dry_run=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h)
      usage
      exit 0
      ;;
    --dry-run)
      dry_run=1
      ;;
    *)
      echo "unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"
ISO_TMP="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}/tmp-scripts-smoke"
ensure_artifacts_dir "$ISO_TMP"
mkdir -p "$ISO_TMP"

fails=()
warns=()
fails_file="$(mktemp "${TMPDIR:-/tmp}/test-scripts-smoke.fail.XXXXXX")"
warns_file="$(mktemp "${TMPDIR:-/tmp}/test-scripts-smoke.warn.XXXXXX")"
trap 'rm -f "$fails_file" "$warns_file"' EXIT
run_probe() {
  local rel="$1"
  [[ -n "$rel" ]] || return 0
  [[ "$rel" == scripts/_lib/* ]] && return 0
  [[ "$rel" == "scripts/test/test-scripts-smoke.sh" ]] && return 0
  local abs="$ROOT_DIR/$rel"
  if ! timeout 8s "$abs" --help >/dev/null 2>&1; then
    printf '%s\n' "$rel --help failed" >>"$fails_file"
    return 0
  fi
  if [[ "${TEST_SCRIPTS_SMOKE_PROBE_DRY_RUN:-0}" == "1" ]] \
    && timeout 8s "$abs" --help 2>&1 | rg -n -- '--dry-run' >/dev/null 2>&1; then
    local per_script_iso="$ISO_TMP/$rel"
    mkdir -p "$per_script_iso"
    if ! ISO_ROOT="$per_script_iso" timeout 8s "$abs" --dry-run >/dev/null 2>&1; then
      printf '%s\n' "$rel --dry-run failed" >>"$warns_file"
    fi
  fi
}
jobs="${SCRIPT_SMOKE_JOBS:-8}"
export ROOT_DIR ISO_TMP fails_file warns_file
export -f run_probe
awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC" \
  | xargs -P "$jobs" -I{} bash -lc 'run_probe "$@"' _ {}

if [[ -s "$fails_file" ]]; then
  while IFS= read -r line; do
    fails+=("$line")
  done < <(sort -u "$fails_file")
fi
if [[ -s "$warns_file" ]]; then
  while IFS= read -r line; do
    warns+=("$line")
  done < <(sort -u "$warns_file")
fi

if [[ ${#fails[@]} -gt 0 ]]; then
  echo "test-scripts-smoke: failures:" >&2
  printf '%s\n' "${fails[@]}" >&2
  exit 1
fi

if [[ "$dry_run" == "1" ]]; then
  if [[ ${#warns[@]} -gt 0 ]]; then
    echo "test-scripts-smoke: warnings:" >&2
    printf '%s\n' "${warns[@]}" >&2
  fi
  echo "test-scripts-smoke: dry-run OK"
  exit 0
fi

if [[ ${#warns[@]} -gt 0 ]]; then
  echo "test-scripts-smoke: warnings:" >&2
  printf '%s\n' "${warns[@]}" >&2
fi

echo "test-scripts-smoke: OK"
