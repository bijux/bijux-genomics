#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
GIT_SHORT_SHA=$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || echo nogit)
ISO_ID=$(date -u +%Y%m%d%H%M%S)-$$-$GIT_SHORT_SHA
ISO_ROOT="$ROOT_DIR/artifacts/isolates/$ISO_ID/target-temp/check-generated-configs"
TMP_ROOT="$ISO_ROOT/tmp"

mkdir -p "$TMP_ROOT"

export CARGO_TARGET_DIR="$ISO_ROOT/target"
export CARGO_HOME="$ISO_ROOT/cargo-home"
export TMPDIR="$TMP_ROOT"
export TMP="$TMP_ROOT"
export TEMP="$TMP_ROOT"

cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir "$ROOT_DIR/domain" --configs-dir "$ROOT_DIR/configs" >/dev/null

git diff --exit-code -- \
  configs/tool_registry.toml \
  configs/stages.toml \
  configs/images.toml
