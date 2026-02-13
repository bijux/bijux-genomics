#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

allowed_dirs=(
  "00-intro"
  "10-architecture"
  "20-science"
  "30-operations"
  "40-policies"
  "50-reference"
  "assets"
  "cli"
  "containers"
  "decisions"
  "overrides"
)

failed=0
while IFS= read -r entry; do
  base="${entry##*/}"
  if [[ -d "$entry" ]]; then
    ok=0
    for d in "${allowed_dirs[@]}"; do
      [[ "$base" == "$d" ]] && ok=1 && break
    done
    if [[ $ok -eq 0 ]]; then
      echo "doc-root-layout: unsupported docs root directory: docs/$base" >&2
      failed=1
    fi
  elif [[ -f "$entry" ]]; then
    if [[ "$base" != "index.md" && "$base" != "DOCS_GRAPH.toml" ]]; then
      echo "doc-root-layout: unsupported docs root file: docs/$base" >&2
      failed=1
    fi
  fi
done < <(find "$ROOT_DIR/docs" -mindepth 1 -maxdepth 1)

if [[ $failed -ne 0 ]]; then
  exit 1
fi

echo "doc-root-layout: OK"
