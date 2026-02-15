#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 '<nextest filter expression>' [runs]" >&2
  echo "Example: $0 'test(mod_contracts_pipeline_rs::pipeline_e2e::pipeline_bam_shotgun_report_snapshot)' 20" >&2
  exit 2
fi

expr="$1"
runs="${2:-20}"

if ! [[ "$runs" =~ ^[0-9]+$ ]] || [[ "$runs" -lt 1 ]]; then
  echo "runs must be a positive integer" >&2
  exit 2
fi

pass=0
fail=0

./bin/isolate sh -ceu '
./bin/require-isolate >/dev/null
export TZ=UTC LC_ALL=C
export CARGO_TARGET_DIR="$ISO_ROOT/target-test"
log_dir="$ISO_ROOT/artifacts/flake-hunt"
mkdir -p "$log_dir"
expr="$1"
runs="$2"
pass=0
fail=0
for i in $(seq 1 "$runs"); do
  echo "[$i/$runs] cargo nextest run --config-file configs/nextest/nextest.toml --profile flake -E $expr"
  if cargo nextest run --config-file configs/nextest/nextest.toml --profile flake -E "$expr" >"$log_dir/last.log" 2>&1; then
    pass=$((pass + 1))
    echo "  PASS"
  else
    fail=$((fail + 1))
    echo "  FAIL"
    sed -n "1,120p" "$log_dir/last.log"
  fi
done
printf "Expression: %s\nRuns: %s\nPassed: %s\nFailed: %s\n" "$expr" "$runs" "$pass" "$fail"
[[ "$fail" -eq 0 ]]
' sh "$expr" "$runs"
