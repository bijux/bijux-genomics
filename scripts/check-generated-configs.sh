#!/bin/sh
set -eu

cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs >/dev/null

git diff --exit-code -- \
  configs/tool_registry.toml \
  configs/stages.toml \
  configs/images.toml
