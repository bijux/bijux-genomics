#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

./bin/isolate sh -ceu './bin/require-isolate >/dev/null; cargo fmt --all -- --check'
