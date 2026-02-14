#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

DOMAIN_DIR="$ROOT_DIR/domain"
[[ -d "$DOMAIN_DIR" ]] || {
  echo "domain layout: missing $DOMAIN_DIR" >&2
  exit 1
}

tmp_files="$(mktemp "${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}/domain-layout.XXXXXX")"
trap 'rm -f "$tmp_files"' EXIT INT TERM

find "$DOMAIN_DIR" -type f | sort > "$tmp_files"

if grep -E '\.tmp$' "$tmp_files" >/dev/null; then
  echo "domain layout: forbidden *.tmp files under domain/" >&2
  grep -E '\.tmp$' "$tmp_files" >&2
  exit 1
fi

status=0
while IFS= read -r file; do
  rel="${file#"$ROOT_DIR/"}"
  ok=0
  case "$rel" in
    domain/*/index.yaml|domain/*/artifacts.yaml|domain/*/metrics.yaml) ok=1 ;;
    domain/*/stages/*.yaml|domain/*/tools/*.yaml) ok=1 ;;
    domain/*/metrics/_schema.yaml|domain/*/artifacts/_schema.yaml) ok=1 ;;
    domain/*/fixtures/*|domain/*/fixtures/*/*|domain/*/fixtures/*/*/*) ok=1 ;;
    domain/*/docs/*|domain/*/docs/*/*) ok=1 ;;
  esac
  if [[ "$ok" -eq 0 ]]; then
    echo "domain layout: unknown file not in allowlist: $rel" >&2
    status=1
  fi
done < "$tmp_files"

if [[ "$status" -ne 0 ]]; then
  exit 1
fi

echo "domain layout: OK"
