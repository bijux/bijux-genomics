#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

errors=0
for req in README.md example.toml bench-suite.toml make.sh golden/plan.json golden/explain.json golden/report.json; do
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
    for req in README.md example.toml bench-suite.toml golden/plan.json golden/explain.json golden/report.json; do
      if [[ ! -f "$dir/$req" ]]; then
        echo "examples structure: $rel missing $req" >&2
        errors=1
      fi
    done

    ex_id="$(awk -F'=' '/^id[[:space:]]*=/{gsub(/"/,"",$2); gsub(/[[:space:]]/,"",$2); print $2; exit}' "$dir/example.toml")"
    if [[ -z "$ex_id" ]]; then
      echo "examples structure: $rel missing id in example.toml" >&2
      errors=1
    fi
    if ! rg -q '^corpus_id\s*=' "$dir/example.toml"; then
      echo "examples structure: $rel example.toml missing corpus_id" >&2
      errors=1
    fi
    if ! rg -q '^mini_supported\s*=' "$dir/example.toml"; then
      echo "examples structure: $rel example.toml missing mini_supported" >&2
      errors=1
    fi
    canon="./scripts/examples/run.sh ${ex_id}"
    canon_count="$(grep -oF "$canon" "$dir/README.md" | wc -l | tr -d ' ')"
    if [[ "$canon_count" -ne 1 ]]; then
      echo "examples structure: $rel README.md must contain exactly one canonical invocation: $canon" >&2
      errors=1
    fi
    if ! rg -q '^## Step 1 Containers$' "$dir/README.md"; then
      echo "examples structure: $rel README.md missing heading '## Step 1 Containers'" >&2
      errors=1
    fi
    if ! rg -q '^## Step 2 Build/Verify$' "$dir/README.md"; then
      echo "examples structure: $rel README.md missing heading '## Step 2 Build/Verify'" >&2
      errors=1
    fi
    if ! rg -q '^## Step 3 Bench$' "$dir/README.md"; then
      echo "examples structure: $rel README.md missing heading '## Step 3 Bench'" >&2
      errors=1
    fi
    if ! rg -q '^## Step 4 Collect/Report$' "$dir/README.md"; then
      echo "examples structure: $rel README.md missing heading '## Step 4 Collect/Report'" >&2
      errors=1
    fi
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
