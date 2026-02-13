#!/usr/bin/env sh
set -eu
LC_ALL=C
export LC_ALL

ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)
mkdir -p "$ROOT_DIR/artifacts"

scripts=$(
  grep -RhoE "scripts/[A-Za-z0-9_./-]+\\.sh" "$ROOT_DIR/Makefile" "$ROOT_DIR/makefiles" \
    | sort -u
)

if [ -z "$scripts" ]; then
  echo "ci-shell-lint: no scripts referenced by Make"
  exit 0
fi

if command -v shellcheck >/dev/null 2>&1; then
  echo "$scripts" | while IFS= read -r rel; do
    [ -n "$rel" ] || continue
    shellcheck "$ROOT_DIR/$rel"
  done
else
  echo "$scripts" | while IFS= read -r rel; do
    [ -n "$rel" ] || continue
    if head -n 1 "$ROOT_DIR/$rel" | grep -q "bash"; then
      bash -n "$ROOT_DIR/$rel"
    else
      sh -n "$ROOT_DIR/$rel"
    fi
  done
fi

echo "$scripts" | while IFS= read -r rel; do
  [ -n "$rel" ] || continue
  file="$ROOT_DIR/$rel"
  head12=$(sed -n '1,12p' "$file")
  # shell loop side-effects are local in POSIX sh; use temp marker files instead.
  if ! printf '%s' "$head12" | grep -Eq "set -euo pipefail|set -eu"; then
    echo "$rel" >> "$ROOT_DIR/artifacts/.ci-shell-lint-strict.tmp"
  fi
  if ! printf '%s' "$head12" | grep -Eq "LC_ALL=C"; then
    echo "$rel" >> "$ROOT_DIR/artifacts/.ci-shell-lint-locale.tmp"
  fi
done

if [ -f "$ROOT_DIR/artifacts/.ci-shell-lint-strict.tmp" ]; then
  echo "ci-shell-lint: missing strict mode:" >&2
  cat "$ROOT_DIR/artifacts/.ci-shell-lint-strict.tmp" >&2
  rm -f "$ROOT_DIR/artifacts/.ci-shell-lint-strict.tmp" "$ROOT_DIR/artifacts/.ci-shell-lint-locale.tmp"
  exit 1
fi
if [ -f "$ROOT_DIR/artifacts/.ci-shell-lint-locale.tmp" ]; then
  echo "ci-shell-lint: missing LC_ALL=C:" >&2
  cat "$ROOT_DIR/artifacts/.ci-shell-lint-locale.tmp" >&2
  rm -f "$ROOT_DIR/artifacts/.ci-shell-lint-strict.tmp" "$ROOT_DIR/artifacts/.ci-shell-lint-locale.tmp"
  exit 1
fi

rm -f "$ROOT_DIR/artifacts/.ci-shell-lint-strict.tmp" "$ROOT_DIR/artifacts/.ci-shell-lint-locale.tmp"
echo "ci-shell-lint: OK"
