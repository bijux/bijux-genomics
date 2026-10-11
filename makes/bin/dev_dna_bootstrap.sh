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
mkdir -p "$target_dir/developer-bootstrap"
messages_path="$(mktemp "$target_dir/developer-bootstrap/build-messages.XXXXXX")"
trap 'rm -f "$messages_path"' 0
cargo build --locked -q -p bijux-dna-dev --bin bijux-dna-dev --target-dir "$target_dir" \
  --message-format=json > "$messages_path"

# A configured target can redirect Cargo output while an old native executable
# remains. Admit Cargo's declared artifact, including a fingerprinted no-op build.
python3 "$(dirname "$0")/developer_build_artifact.py" "$messages_path" "$bin_path" \
  crates/bijux-dna-dev/Cargo.toml

if [ ! -x "$bin_path" ]; then
  echo "developer bootstrap did not produce the requested executable: $bin_path" >&2
  exit 1
fi
