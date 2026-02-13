#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SNAP="$ROOT_DIR/docs/cli/command_snapshot.txt"
[[ -f "$SNAP" ]] || { echo "missing $SNAP" >&2; exit 1; }

allowed_subcommands="$(awk '/^  [a-z]/{print $1}' "$SNAP" | tr '\n' ' ')"
errors=0

while IFS= read -r ex_toml; do
  md="$(dirname "$ex_toml")/README.md"
  [[ -f "$md" ]] || continue
  while IFS= read -r cmd; do
    clean="${cmd#\`}"
    clean="${clean%\`}"
    if [[ "$clean" == bijux* ]]; then
      sub="$(echo "$clean" | awk '{print $2}')"
      if [[ -z "$sub" ]]; then
        continue
      fi
      if ! grep -qw "$sub" <<< "$allowed_subcommands"; then
        echo "examples cli snapshot: ${md#"$ROOT_DIR/"} uses '$clean' but '$sub' not in docs/cli/command_snapshot.txt" >&2
        errors=1
      fi
    fi
  done < <(rg -No '\`[^`]+\`' "$md" | cut -d: -f3)
done < <(find "$ROOT_DIR/examples" -type f -name example.toml | sort)

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi
echo "examples cli snapshot: OK"
