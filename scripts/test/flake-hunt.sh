#!/usr/bin/env bash
set -euo pipefail
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

for i in $(seq 1 "$runs"); do
  echo "[$i/$runs] cargo nextest run --config-file nextest.toml --profile flake -E $expr"
  if cargo nextest run --config-file nextest.toml --profile flake -E "$expr" >/tmp/flake-hunt-last.log 2>&1; then
    pass=$((pass + 1))
    echo "  PASS"
  else
    fail=$((fail + 1))
    echo "  FAIL"
    sed -n '1,120p' /tmp/flake-hunt-last.log
  fi
done

echo
printf 'Expression: %s\n' "$expr"
printf 'Runs:       %s\n' "$runs"
printf 'Passed:     %s\n' "$pass"
printf 'Failed:     %s\n' "$fail"

if [[ "$fail" -gt 0 ]]; then
  exit 1
fi
