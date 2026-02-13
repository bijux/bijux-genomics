#!/usr/bin/env sh
set -eu
LC_ALL=C
export LC_ALL

if ! command -v rg >/dev/null 2>&1; then
  echo "raw-cargo-policy: ripgrep (rg) is required" >&2
  exit 127
fi

matches=$(rg -n "cargo (fmt|clippy|test|run|deny|nextest|llvm-cov|insta|build|check|doc|install)\\b" scripts || true)
if [ -z "$matches" ]; then
  echo "raw-cargo-policy(scripts): OK"
  exit 0
fi

violations=""
while IFS= read -r row; do
  [ -n "$row" ] || continue
  file=$(printf '%s' "$row" | cut -d: -f1)
  line=$(printf '%s' "$row" | cut -d: -f3-)

  case "$line" in
    *"./bin/isolate cargo "*|*"# Regenerate with: cargo run"*)
      continue
      ;;
  esac

  if rg -n 'exec ./bin/isolate "\$0" "\$@"' "$file" >/dev/null 2>&1; then
    continue
  fi

  violations="${violations}${row}\n"
done <<ROWS
$matches
ROWS

if [ -n "$violations" ]; then
  echo "raw-cargo-policy(scripts): direct cargo invocation found; scripts must self-isolate or use ./bin/isolate cargo ..." >&2
  printf '%b' "$violations" >&2
  exit 1
fi

echo "raw-cargo-policy(scripts): OK"
