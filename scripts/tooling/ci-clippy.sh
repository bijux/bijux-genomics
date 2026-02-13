#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

cargo_build_jobs="${CARGO_BUILD_JOBS:-8}"

./bin/isolate sh -ceu "./bin/require-isolate >/dev/null; CARGO_BUILD_JOBS='${cargo_build_jobs}' cargo clippy --workspace --all-targets --all-features -- -D warnings"
