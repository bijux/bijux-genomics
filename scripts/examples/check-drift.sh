#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'EOF'
Usage: scripts/examples/check-drift.sh <example-id>
EOF
}

if [[ "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

[[ $# -eq 1 ]] || { usage >&2; exit 2; }
id="$1"

ex_dir="$(find "$ROOT_DIR/examples" -type f -name example.toml -print | while read -r f; do
  if rg -q "^id\\s*=\\s*\"${id}\"\\s*$" "$f"; then dirname "$f"; break; fi
done)"
[[ -n "$ex_dir" ]] || { echo "unknown example id: $id" >&2; exit 1; }

"$ROOT_DIR/scripts/examples/run.sh" "$id" >/dev/null

art_dir="${ISO_ROOT:-$ROOT_DIR/artifacts}/examples/${id}"
diff -u "$ex_dir/golden/plan.json" "$art_dir/plan.json" >/dev/null || {
  echo "example drift: plan mismatch for $id" >&2
  exit 1
}
diff -u "$ex_dir/golden/explain.json" "$art_dir/explain.json" >/dev/null || {
  echo "example drift: explain mismatch for $id" >&2
  exit 1
}
echo "example drift: OK ($id)"
