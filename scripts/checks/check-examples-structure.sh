#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

errors=0
for req in README.md example.toml bench-suite.toml golden/plan.json golden/explain.json; do
  if [[ ! -f "$ROOT_DIR/examples/_template/$req" ]]; then
    echo "examples structure: examples/_template missing $req" >&2
    errors=1
  fi
done

for dir in "$ROOT_DIR"/examples/* "$ROOT_DIR"/examples/*/* "$ROOT_DIR"/examples/*/*/*; do
  [[ -d "$dir" ]] || continue
  rel="${dir#"$ROOT_DIR/"}"
  case "$rel" in
    examples/_template|examples/data|examples/data/*|examples/data/*/*|examples/fastq|examples/bam|examples/vcf) continue ;;
  esac
  if [[ -f "$dir/example.toml" ]]; then
    for req in README.md example.toml bench-suite.toml golden/plan.json golden/explain.json; do
      if [[ ! -f "$dir/$req" ]]; then
        echo "examples structure: $rel missing $req" >&2
        errors=1
      fi
    done
  fi
done

"$ROOT_DIR/scripts/examples/check-index.sh" || errors=1

if rg -n "bijux-dna-data" "$ROOT_DIR/examples" "$ROOT_DIR/docs/50-reference/EXAMPLES.md" >/dev/null 2>&1; then
  echo "examples structure: found stale 'bijux-dna-data' references; use examples/data" >&2
  errors=1
fi

if [[ "$errors" -ne 0 ]]; then
  exit 1
fi
echo "examples structure: OK"
