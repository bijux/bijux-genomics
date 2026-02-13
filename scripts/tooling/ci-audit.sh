#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

./bin/isolate sh -ceu '
./bin/require-isolate >/dev/null
command -v cargo-deny >/dev/null 2>&1 || { echo "missing required tool: cargo-deny"; echo "install once: cargo install cargo-deny --locked"; exit 1; }
cargo deny check
'
