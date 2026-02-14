#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'USAGE'
Usage: scripts/test/test-scripts-smoke.sh [--help] [--dry-run]
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
while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  [[ "$rel" == scripts/_lib/* ]] && continue
  [[ "$rel" == "scripts/test/test-scripts-smoke.sh" ]] && continue
  abs="$ROOT_DIR/$rel"
  if ! timeout 8s "$abs" --help >/dev/null 2>&1; then
    fails+=("$rel --help failed")
    continue
  fi
  # dry-run probe: prefer explicit --dry-run, else --help probe is the dry-run.
  if timeout 8s "$abs" --help 2>&1 | rg -n -- '--dry-run' >/dev/null 2>&1; then
    if ! ISO_ROOT="$ISO_TMP" timeout 8s "$abs" --dry-run >/dev/null 2>&1; then
      warns+=("$rel --dry-run failed")
    fi
  fi
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

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
