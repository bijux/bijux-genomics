#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)

if ! command -v rg >/dev/null 2>&1; then
  echo "generated-config-headers: ripgrep (rg) is required but not found in PATH" >&2
  exit 127
fi

fail() {
  echo "generated-config-headers: $*" >&2
  exit 1
}

check_file() {
  file="$1"
  [ -f "$file" ] || fail "missing generated config: $file"
  head4=$(sed -n '1,4p' "$file")
  echo "$head4" | rg -q '^# GENERATED - DO NOT EDIT - source: domain/\*\*$' \
    || fail "$file missing generated header line"
  echo "$head4" | rg -q '^# source_commit: [0-9a-f]{40}$' \
    || fail "$file missing source commit hash header"
  echo "$head4" | rg -q '^# domain_schema_version: bijux.domain.v1$' \
    || fail "$file missing domain schema version header"
  echo "$head4" | rg -q '^# Regenerate with: cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs$' \
    || fail "$file missing regenerate command header"
}

check_file "$ROOT_DIR/configs/ci/tool_registry.toml"
check_file "$ROOT_DIR/configs/ci/tool_registry_experimental.toml"
check_file "$ROOT_DIR/configs/ci/required_tools.toml"
check_file "$ROOT_DIR/configs/ci/stages.toml"
check_file "$ROOT_DIR/configs/ci/images.toml"

echo "generated-config-headers: OK"
