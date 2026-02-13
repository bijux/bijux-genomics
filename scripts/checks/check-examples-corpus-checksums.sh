#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'EOF'
Usage: scripts/checks/check-examples-corpus-checksums.sh
EOF
}

if [[ "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

errors=0
for corpus_dir in "$ROOT_DIR"/examples/data/corpus-*; do
  [[ -d "$corpus_dir" ]] || continue
  rel="${corpus_dir#"$ROOT_DIR/"}"
  checksums="$corpus_dir/CHECKSUMS.sha256"
  if [[ ! -f "$checksums" ]]; then
    echo "corpus checksums: missing ${checksums#"$ROOT_DIR/"}" >&2
    errors=1
    continue
  fi

  if [[ -d "$corpus_dir/raw" ]] && ! rg -q '(^|[[:space:]])raw/' "$checksums"; then
    echo "corpus checksums: $rel missing raw/ entries in CHECKSUMS.sha256" >&2
    errors=1
  fi
  if [[ -d "$corpus_dir/normalized" ]] && ! rg -q '(^|[[:space:]])normalized/' "$checksums"; then
    echo "corpus checksums: $rel missing normalized/ entries in CHECKSUMS.sha256" >&2
    errors=1
  fi

  (
    cd "$corpus_dir"
    if ! shasum -a 256 -c CHECKSUMS.sha256 >/dev/null; then
      echo "corpus checksums: mismatch in $rel" >&2
      exit 1
    fi
  ) || errors=1
done

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi

echo "examples corpus checksums: OK"
