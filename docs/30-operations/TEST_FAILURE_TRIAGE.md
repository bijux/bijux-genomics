# Test Failure Triage

This document defines the default buckets used to triage CI/local test failures.

## Buckets

- `guardrails`: naming/lint/guardrail policy checks (for example `policy_test_names_are_consistent`, workspace-lints coverage).
- `snapshots`: snapshot drift or insta failures (`*.snap` changes, schema drift).
- `ssot-registry`: registry completeness/binding/SSOT contract failures.
- `apptainer-policy`: Apptainer image, smoke, and container policy failures.
- `spawn-policy`: process-spawn restrictions and runner boundary checks.
- `other`: anything not matched by known patterns.

## Usage

1. Save the full failing test output:
   - `mkdir -p artifacts/test-logs`
   - `make test | tee artifacts/test-logs/<timestamp>.log`
2. Point `latest.log` to the run you are triaging:
   - `cp artifacts/test-logs/<timestamp>.log artifacts/test-logs/latest.log`
3. Run:
   - `make test-triage`

## Notes

- `make test-triage` is heuristic and intended for fast diagnosis.
- Final source of truth remains `cargo nextest` output and individual failing test logs.
