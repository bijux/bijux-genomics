#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

failed=0
while IFS= read -r p; do
  [[ -n "$p" ]] || continue
  f="$ROOT_DIR/$p"
  if rg -n '(>|>>|cp |mv |mkdir -p|rm -rf)\s*/(tmp|var|opt|usr|etc|home|Users)\b' "$f" >/dev/null 2>&1; then
    echo "output-roots: forbidden absolute write pattern in $p" >&2
    failed=1
  fi
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$ROOT_DIR/scripts/SUPPORTED.toml")
[[ $failed -eq 0 ]] || exit 1
echo "output-roots(static): OK"

sentinel_base="$ROOT_DIR/.sentinel-readonly"
rm -rf "$sentinel_base"
mkdir -p "$sentinel_base"
chmod 555 "$sentinel_base"
if touch "$sentinel_base/forbidden" 2>/dev/null; then
  echo "output-roots(runtime): sentinel unexpectedly writable" >&2
  chmod 755 "$sentinel_base"
  rm -rf "$sentinel_base"
  exit 1
fi
chmod 755 "$sentinel_base"
rm -rf "$sentinel_base"

echo "output-roots(runtime): OK"
