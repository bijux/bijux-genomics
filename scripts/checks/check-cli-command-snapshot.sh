#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

snap_root="$ROOT_DIR/docs/cli"
root_snap="$snap_root/command_snapshot.txt"
dna_snap="$snap_root/release_help_snapshot.txt"
[[ -f "$root_snap" ]] || { echo "cli snapshot: missing $root_snap" >&2; exit 1; }
[[ -f "$dna_snap" ]] || { echo "cli snapshot: missing $dna_snap" >&2; exit 1; }

tmp_root="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}/cli-snapshots"
ensure_artifacts_dir "$tmp_root"
mkdir -p "$tmp_root"
actual_root="$tmp_root/command_snapshot.actual.txt"
actual_dna="$tmp_root/release_help_snapshot.actual.txt"

"$ROOT_DIR/bin/isolate" sh -ceu "
  export CARGO_TARGET_DIR=\"\$ISO_ROOT/target-cli-snapshot\"
  cargo run --quiet --bin bijux -- --help > \"$actual_root\"
  cargo run --quiet --bin bijux -- dna --help > \"$actual_dna\"
"

if ! diff -u "$root_snap" "$actual_root" >/dev/null; then
  echo "cli snapshot drift: docs/cli/command_snapshot.txt differs from controlled bijux --help output" >&2
  exit 1
fi
if ! diff -u "$dna_snap" "$actual_dna" >/dev/null; then
  echo "cli snapshot drift: docs/cli/release_help_snapshot.txt differs from controlled bijux dna --help output" >&2
  exit 1
fi

echo "cli snapshot: OK"
