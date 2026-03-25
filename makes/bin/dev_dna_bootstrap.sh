#!/usr/bin/env sh
set -eu

bin_path="${1:-artifacts/target/debug/bijux-dna-dev}"
needs_build=0

if [ ! -x "$bin_path" ]; then
  needs_build=1
else
  for path in Cargo.toml Cargo.lock $(find crates/bijux-dna-dev -type f); do
    if [ "$path" -nt "$bin_path" ]; then
      needs_build=1
      break
    fi
  done
fi

if [ "$needs_build" -eq 1 ]; then
  cargo build -q -p bijux-dna-dev
fi
