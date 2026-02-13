#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

errors=0
policy="$ROOT_DIR/examples/POLICY.md"
if [[ ! -f "$policy" ]]; then
  echo "notebook policy: missing examples/POLICY.md" >&2
  errors=1
elif ! rg -q "Notebook Optional Path Rule" "$policy"; then
  echo "notebook policy: examples/POLICY.md missing 'Notebook Optional Path Rule' section" >&2
  errors=1
fi

while IFS= read -r nb; do
  ex_dir="$(dirname "$nb")"
  readme="$ex_dir/README.md"
  if [[ ! -f "$readme" ]]; then
    echo "notebook policy: ${nb#"$ROOT_DIR/"} requires README.md in same directory" >&2
    errors=1
    continue
  fi
  if ! rg -qi "optional notebook" "$readme"; then
    echo "notebook policy: ${readme#"$ROOT_DIR/"} must state notebook is optional" >&2
    errors=1
  fi
  if ! rg -qi "reproducible from cli|reproducible from command line" "$readme"; then
    echo "notebook policy: ${readme#"$ROOT_DIR/"} must state outputs are reproducible from CLI" >&2
    errors=1
  fi
done < <(find "$ROOT_DIR/examples" -type f -name '*.ipynb' | sort)

if rg -n '\.ipynb' "$ROOT_DIR/scripts/examples/run.sh" >/dev/null 2>&1; then
  echo "notebook policy: scripts/examples/run.sh must not depend on notebooks" >&2
  errors=1
fi

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi
echo "examples notebook policy: OK"
