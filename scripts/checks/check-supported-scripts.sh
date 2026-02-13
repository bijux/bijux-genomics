#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
ALLOWLIST="$ROOT_DIR/scripts/checks/supported_scripts.txt"

if [[ ! -f "$ALLOWLIST" ]]; then
  echo "supported-scripts: missing allowlist: $ALLOWLIST" >&2
  exit 1
fi

allowlist=$(sed '/^\s*$/d' "$ALLOWLIST" | sort -u)
while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  [[ -f "$ROOT_DIR/$rel" ]] || { echo "supported-scripts: allowlisted file not found: $rel" >&2; exit 1; }
done <<EOF
$allowlist
EOF

missing=()
referenced=$(grep -RhoE 'scripts/[A-Za-z0-9_./-]+\.sh' "$ROOT_DIR/Makefile" "$ROOT_DIR/makefiles" | sort -u)
while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  if ! grep -qx "$rel" "$ALLOWLIST"; then
    missing+=("$rel")
  fi
done <<EOF
$referenced
EOF

if [[ ${#missing[@]} -gt 0 ]]; then
  echo "supported-scripts: scripts referenced by make/CI but not allowlisted:" >&2
  printf '%s\n' "${missing[@]}" >&2
  exit 1
fi

echo "supported-scripts: OK"
