#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT_DIR"

if ! ./bin/require-isolate >/dev/null 2>&1; then
  exec ./bin/isolate "$0" "$@"
fi

cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir "$ROOT_DIR/domain" --configs-dir "$ROOT_DIR/configs" >/dev/null

git diff --exit-code -- \
  configs/ci/tool_registry.toml \
  configs/ci/tool_registry_experimental.toml \
  configs/ci/required_tools.toml \
  configs/ci/stages.toml \
  configs/ci/images.toml
