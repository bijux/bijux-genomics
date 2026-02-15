#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

viol=()
allowlist=(
  "crates/bijux-dna-api/src/runtime/execution_kernel_support.rs:126"
  "crates/bijux-dna-api/src/runtime/execution_kernel_support.rs:127"
  "crates/bijux-dna-api/src/runtime/execution_kernel_support.rs:128"
  "crates/bijux-dna-api/src/runtime/execution_kernel_support.rs:129"
  "crates/bijux-dna-api/src/runtime/execution_kernel_support.rs:693"
  "crates/bijux-dna-api/src/runtime/execution_kernel_support.rs:702"
  "crates/bijux-dna-runner/src/backend/docker/executor.rs:367"
)
while IFS= read -r file; do
  rel="${file#"$ROOT_DIR/"}"
  while IFS= read -r hit; do
    line="${hit%%:*}"
    loc="${rel}:${line}"
    allowed=0
    for rule in "${allowlist[@]}"; do
      if [[ "$loc" == "$rule" ]]; then
        allowed=1
        break
      fi
    done
    if [[ "$allowed" -eq 0 ]]; then
      viol+=("$rel:$line uses hidden system tmp path; use runtime tmp root contracts")
    fi
  done < <(rg -n '(^|[[:space:]"'"'"'=])(/tmp|/var/tmp)(/|$)' "$file" || true)
done < <(find \
  "$ROOT_DIR/crates/bijux-dna-api/src/runtime" \
  "$ROOT_DIR/crates/bijux-dna-api/src/internal/handlers/cross" \
  "$ROOT_DIR/crates/bijux-dna-runner/src" \
  -type f -name '*.rs' -print)

if ((${#viol[@]} > 0)); then
  printf '%s\n' "check-hidden-tmp-usage: FAILED"
  printf ' - %s\n' "${viol[@]}"
  exit 1
fi

echo "check-hidden-tmp-usage: OK"
