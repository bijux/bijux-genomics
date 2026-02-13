#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" || "${1:-}" == "--dry-run" || "${1:-}" == "--verbose" ]]; then
  cat <<'USAGE'
Usage: scripts/checks/check-assets-large-file-allowlist.sh
USAGE
  exit 0
fi

allow="$ROOT_DIR/assets/LARGE_FILE_ALLOWLIST.txt"
[[ -f "$allow" ]] || { echo "assets-large-file-allowlist: missing assets/LARGE_FILE_ALLOWLIST.txt" >&2; exit 1; }

errors=0
while IFS= read -r line; do
  s="$(printf '%s' "$line" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
  [[ -z "$s" || "${s#\#}" != "$s" ]] && continue
  # contract: path | reason=<...> | owner=<...> | expiry=YYYY-MM-DD
  if ! grep -Eq '^[^|]+[[:space:]]*\|[[:space:]]*reason=[^|]+[[:space:]]*\|[[:space:]]*owner=[^|]+[[:space:]]*\|[[:space:]]*expiry=[0-9]{4}-[0-9]{2}-[0-9]{2}[[:space:]]*$' <<<"$s"; then
    echo "assets-large-file-allowlist: invalid entry format: $s" >&2
    errors=1
  fi
done < "$allow"

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi

echo "assets-large-file-allowlist: OK"
