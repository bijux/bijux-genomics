#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

example_id="$(awk -F'=' '/^id[[:space:]]*=/{gsub(/"/,"",$2); gsub(/[[:space:]]/,"",$2); print $2; exit}' "$ROOT_DIR/examples/fastq/qc-pre-bench/example.toml")"
[[ -n "$example_id" ]] || { echo "examples runner contract: cannot resolve baseline example id" >&2; exit 1; }

run_once() {
  local tag="$1"
  ISO_TAG="$tag" "$ROOT_DIR/bin/isolate" sh -ceu "
    ./scripts/examples/run.sh ${example_id} >/dev/null
  "
}

run_once "examples-runner-a"
run_once "examples-runner-b"

a_root="$(ISO_TAG=examples-runner-a "$ROOT_DIR/bin/isolate" --print-root)"
b_root="$(ISO_TAG=examples-runner-b "$ROOT_DIR/bin/isolate" --print-root)"
a_dir="$a_root/examples/${example_id}"
b_dir="$b_root/examples/${example_id}"

for jf in plan.json explain.json report.json; do
  if ! diff -u "$a_dir/$jf" "$b_dir/$jf" >/dev/null; then
    echo "examples runner contract: non-deterministic $jf for ${example_id}" >&2
    exit 1
  fi
done

echo "examples runner contract: OK"
