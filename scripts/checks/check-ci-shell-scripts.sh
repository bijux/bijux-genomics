#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
ALLOWLIST="$ROOT_DIR/scripts/checks/supported_scripts.txt"

if [[ ! -f "$ALLOWLIST" ]]; then
  echo "ci-shell-lint: missing allowlist: $ALLOWLIST" >&2
  exit 1
fi

scripts=$(sed '/^\s*$/d' "$ALLOWLIST" | rg '\.sh$' | sort -u)
if [[ -z "$scripts" ]]; then
  echo "ci-shell-lint: no shell scripts in allowlist"
  exit 0
fi

if command -v shellcheck >/dev/null 2>&1; then
  while IFS= read -r rel; do
    [[ -n "$rel" ]] || continue
    shellcheck "$ROOT_DIR/$rel"
  done <<EOF
$scripts
EOF
else
  while IFS= read -r rel; do
    [[ -n "$rel" ]] || continue
    bash -n "$ROOT_DIR/$rel"
  done <<EOF
$scripts
EOF
fi

missing=()
while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  file="$ROOT_DIR/$rel"
  first="$(sed -n '1,6p' "$file")"
  first_line="$(printf '%s\n' "$first" | sed -n '1p')"
  if [[ "$first_line" != '#!/usr/bin/env bash' ]]; then
    missing+=("$rel: missing '#!/usr/bin/env bash'")
  fi
  if ! printf '%s\n' "$first" | rg -qx 'set -euo pipefail'; then
    missing+=("$rel: missing 'set -euo pipefail'")
  fi
  if ! printf '%s\n' "$first" | grep -Fx "IFS=\$'\\n\\t'" >/dev/null 2>&1; then
    missing+=("$rel: missing \"IFS=\\$'\\n\\t'\"")
  fi
done <<EOF
$scripts
EOF

if [[ ${#missing[@]} -gt 0 ]]; then
  echo "ci-shell-lint: strict mode header violations:" >&2
  printf '%s\n' "${missing[@]}" >&2
  exit 1
fi

echo "ci-shell-lint: OK"
