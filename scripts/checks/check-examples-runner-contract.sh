#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

example_id="$(awk -F'=' '/^id[[:space:]]*=/{gsub(/"/,"",$2); gsub(/[[:space:]]/,"",$2); print $2; exit}' "$ROOT_DIR/examples/fastq/qc-pre-bench/example.toml")"
[[ -n "$example_id" ]] || { echo "examples runner contract: cannot resolve baseline example id" >&2; exit 1; }

run_once() {
  local run_label="$1"
  local output_dir="$ROOT_DIR/artifacts/examples/${example_id}"
  rm -rf "$output_dir"
  ARTIFACT_ROOT="$ROOT_DIR/artifacts" run_with_artifact_env ./scripts/examples/run.sh "${example_id}" >/dev/null
  rm -rf "$ROOT_DIR/artifacts/examples/${example_id}.${run_label}"
  cp -R "$output_dir" "$ROOT_DIR/artifacts/examples/${example_id}.${run_label}"
}

run_once "examples-runner-a"
run_once "examples-runner-b"

a_dir="$ROOT_DIR/artifacts/examples/${example_id}.examples-runner-a"
b_dir="$ROOT_DIR/artifacts/examples/${example_id}.examples-runner-b"

for jf in plan.json explain.json report.json; do
  if ! diff -u "$a_dir/$jf" "$b_dir/$jf" >/dev/null; then
    echo "examples runner contract: non-deterministic $jf for ${example_id}" >&2
    exit 1
  fi
done

echo "examples runner contract: OK"
