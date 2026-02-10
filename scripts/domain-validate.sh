#!/bin/sh
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
isolate_root="${DOMAIN_VALIDATE_ISOLATE_ROOT:-$repo_root/artifacts/isolates/target-temp/domain-validate}"
stamp="$(date -u +%Y%m%d%H%M%S)-$$"
run_root="$isolate_root/$stamp"
tmp_dir="$run_root/tmp"

mkdir -p "$tmp_dir"

if [ -z "${CARGO_TARGET_DIR:-}" ]; then
  export CARGO_TARGET_DIR="$run_root/target"
fi
if [ -z "${CARGO_HOME:-}" ]; then
  export CARGO_HOME="$run_root/cargo-home"
fi
export TMPDIR="$tmp_dir"
export TMP="$tmp_dir"
export TEMP="$tmp_dir"

cargo run --bin bijux-dna -- domain validate --domain-dir "$repo_root/domain"
