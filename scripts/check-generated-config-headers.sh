#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

fail() {
  echo "generated-config-headers: $*" >&2
  exit 1
}

check_file() {
  file="$1"
  [ -f "$file" ] || fail "missing generated config: $file"
  head3=$(sed -n '1,3p' "$file")
  echo "$head3" | rg -q '^# GENERATED - DO NOT EDIT - source: domain/\*\*$' \
    || fail "$file missing generated header line"
  echo "$head3" | rg -q '^# source_commit: [0-9a-f]{40}$' \
    || fail "$file missing source commit hash header"
  echo "$head3" | rg -q '^# Regenerate with: cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs$' \
    || fail "$file missing regenerate command header"
}

check_file "$ROOT_DIR/configs/tool_registry.toml"
check_file "$ROOT_DIR/configs/stages.toml"
check_file "$ROOT_DIR/configs/images.toml"

echo "generated-config-headers: OK"
