#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" || "${1:-}" == "--dry-run" || "${1:-}" == "--verbose" ]]; then
  cat <<'USAGE'
Usage: scripts/checks/check-gitignore-contract.sh
USAGE
  exit 0
fi

path="$ROOT_DIR/.gitignore"
[[ -f "$path" ]] || { echo "gitignore-contract: missing .gitignore" >&2; exit 1; }

required=(
  "/target/"
  "**/target/"
  "/artifacts/"
  "/artifacts/**"
  "*.tmp"
)

viol=()
for p in "${required[@]}"; do
  if ! rg -q "^${p//\*/\\*}$" "$path"; then
    viol+=("missing required pattern: $p")
  fi
done

# Disallow unignoring target/ paths except explicit artifacts/isolate contract.
while IFS= read -r line; do
  [[ "$line" =~ ^!.*target ]] || continue
  if [[ "$line" != "!/artifacts/isolate/**" && "$line" != "!/artifacts/isolate/" ]]; then
    viol+=("forbidden target unignore pattern: $line")
  fi
done < "$path"

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "gitignore-contract: FAILED" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "gitignore-contract: OK"
