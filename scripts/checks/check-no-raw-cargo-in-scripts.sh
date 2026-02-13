#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
ALLOWLIST="$ROOT_DIR/scripts/checks/supported_scripts.txt"

if ! command -v rg >/dev/null 2>&1; then
  echo "raw-cargo-policy: ripgrep (rg) is required" >&2
  exit 127
fi

violations=""
while IFS= read -r rel; do
  [[ "$rel" == *.sh ]] || continue
  [[ "$rel" == "scripts/checks/check-no-raw-cargo-in-scripts.sh" ]] && continue
  file="$ROOT_DIR/$rel"
  [[ -f "$file" ]] || continue
  if [[ "$rel" == scripts/tooling/ci-*.sh ]]; then
    continue
  fi

  matches=$(rg -Hn "cargo (fmt|clippy|test|run|deny|nextest|llvm-cov|insta|build|check|doc|install)\\b" "$file" || true)
  while IFS= read -r row; do
    [[ -n "$row" ]] || continue
    line=$(printf '%s' "$row" | cut -d: -f3-)

    if [[ "$line" == *"./bin/isolate cargo "* ]]; then
      continue
    fi
    if [[ "$line" == *"Regenerate with: cargo run"* ]]; then
      continue
    fi

    if rg -n 'exec ./bin/isolate "\$0" "\$@"' "$file" >/dev/null 2>&1; then
      continue
    fi
    violations+="$row\n"
  done <<ROWS
$matches
ROWS

  if [[ "$rel" != scripts/containers/* ]]; then
    container_matches=$(rg -Hn "(^|[[:space:];|&])(docker|apptainer)([[:space:]]|$)" "$file" || true)
    if [[ -n "$container_matches" ]]; then
      violations+="$container_matches\n"
    fi
  fi
done < <(sed '/^\s*$/d' "$ALLOWLIST")

if [[ -n "$violations" ]]; then
  echo "raw-tooling-policy(scripts): direct cargo/docker/apptainer invocation violation found" >&2
  printf '%b' "$violations" >&2
  exit 1
fi

echo "raw-cargo-policy(scripts): OK"
