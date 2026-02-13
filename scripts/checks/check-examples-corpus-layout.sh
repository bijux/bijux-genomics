#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

errors=0
for corpus_dir in "$ROOT_DIR"/examples/data/corpus-*; do
  [[ -d "$corpus_dir" ]] || continue
  if [[ -d "$corpus_dir/raw" ]]; then
    if [[ ! -d "$corpus_dir/normalized" ]]; then
      echo "corpus layout: ${corpus_dir#"$ROOT_DIR/"} has raw/ but missing normalized/" >&2
      errors=1
    fi
    if [[ ! -f "$corpus_dir/NORMALIZE.md" ]]; then
      echo "corpus layout: ${corpus_dir#"$ROOT_DIR/"} has raw/ but missing NORMALIZE.md" >&2
      errors=1
    fi
  fi
done

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi
echo "examples corpus layout: OK"
