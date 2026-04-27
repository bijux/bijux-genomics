# Test Failure Triage

This document defines the default buckets used to triage CI/local test failures from
[CI.md](CI.md).

## What
Defines standard failure buckets and a reproducible triage workflow for local/CI runs.

## Why
Makes repeated failure classes easy to route to the right subsystem owner.

## Non-goals
- Replacing full nextest logs as source of truth.
- Auto-fixing failed tests.

## Contracts
- Every failing run should be saved under `artifacts/test-logs/`.
- Buckets are heuristic labels; decisions still require reading failing tests.

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
   - The governed wrapper lives in [makes/cargo.mk](../../makes/cargo.mk) and the command
     inventory is published in
     [crates/bijux-dna-dev/docs/COMMANDS.md](../../crates/bijux-dna-dev/docs/COMMANDS.md).

## Notes

- `make test-triage` is heuristic and intended for fast diagnosis.
- Final source of truth remains the `cargo nextest` surface configured in
  [configs/rust/nextest.toml](../../configs/rust/nextest.toml) and individual failing test logs.

## Examples
- `snapshot` failures from `.snap.new` drift.
- `guardrails` failures from oversized files or module layout limits.

## Failure modes
- Misclassification when a failure spans multiple policy classes.
- Stale `latest.log` causing misleading triage output.
