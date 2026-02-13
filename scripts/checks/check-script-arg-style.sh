#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"

viol=()
positional_dispatch_allowlist=(
  "scripts/_lib/common.sh"
  "scripts/run.sh"
  "scripts/smoke/run.sh"
  "scripts/tooling/benchmarks.sh"
  "scripts/tooling/cargo-targets.sh"
  "scripts/containers/make.sh"
  "scripts/assets/make.sh"
  "scripts/checks/make.sh"
  "scripts/docs/make.sh"
  "scripts/domain/make.sh"
  "scripts/examples/make.sh"
  "scripts/hpc/make.sh"
  "scripts/lab/make.sh"
  "scripts/smoke/make.sh"
  "scripts/test/make.sh"
  "scripts/tooling/make.sh"
  "scripts/containers/lint.sh"
  "scripts/containers/smoke-docker-amd64.sh"
  "scripts/test/toy_runs.sh"
  "scripts/tooling/coverage_summary.sh"
)
while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  file="$ROOT_DIR/$rel"
  [[ -f "$file" ]] || continue

  for allowed in "${positional_dispatch_allowlist[@]}"; do
    if [[ "$rel" == "$allowed" ]]; then
      continue 2
    fi
  done

  # Trivial one-arg scripts are allowed to remain positional.
  if rg -n '"\$[2-9]"|\$\{[2-9]\}|"\$@"|"\$#"' "$file" >/dev/null 2>&1; then
    if ! rg -n -- '--[a-zA-Z0-9_-]+' "$file" >/dev/null 2>&1; then
      viol+=("$rel: uses multi-arg/variadic positional style without --flags")
    fi
  fi

done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "script-arg-style: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "script-arg-style: OK"
