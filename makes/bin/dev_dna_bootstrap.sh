#!/usr/bin/env sh
set -eu

bin_path="${1:-artifacts/target/debug/bijux-dev-dna}"
needs_build=0

if [ ! -x "$bin_path" ]; then
  needs_build=1
else
  for path in Cargo.toml Cargo.lock $(find crates/bijux-dev-dna -type f); do
    if [ "$path" -nt "$bin_path" ]; then
      needs_build=1
      break
    fi
  done
fi

if [ "$needs_build" -eq 1 ]; then
  cargo build -q -p bijux-dev-dna
fi
