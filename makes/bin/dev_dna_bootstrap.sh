#!/usr/bin/env sh
set -eu

bin_path="${1:-${CARGO_TARGET_DIR:-artifacts/target}/debug/bijux-dna-dev}"
profile_dir="$(dirname "$bin_path")"
if [ "$(basename "$bin_path")" != "bijux-dna-dev" ] ||
  [ "$(basename "$profile_dir")" != "debug" ]; then
  echo "developer bootstrap requires a debug/bijux-dna-dev output path: $bin_path" >&2
  exit 2
fi

# Cargo fingerprints include transitive sources, features and build inputs. A
# binary mtime compared only with this crate cannot establish that it is current.
target_dir="$(dirname "$profile_dir")"
cargo build --locked -q -p bijux-dna-dev --bin bijux-dna-dev --target-dir "$target_dir"

if [ ! -x "$bin_path" ]; then
  echo "developer bootstrap did not produce the requested executable: $bin_path" >&2
  exit 1
fi
