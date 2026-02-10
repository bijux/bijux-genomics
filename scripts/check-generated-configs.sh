#!/bin/sh
set -eu

cargo run -p bijux-dna-environment-qa --bin compile_domain_configs -- --domain-dir domain --configs-dir configs >/dev/null

git diff --exit-code -- configs/tool_registry.toml configs/images.toml configs/stages.toml
