#!/usr/bin/env sh
set -eu

./bin/isolate cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs
