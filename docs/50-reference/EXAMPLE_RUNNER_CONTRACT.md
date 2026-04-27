# EXAMPLE_RUNNER_CONTRACT

## Purpose
Define required outputs for any example runner execution.

## Scope
Applies to scripts and commands that run curated examples and sample workflows listed in
[examples/index.yaml](../../examples/index.yaml).

## Non-goals
- Defining tool-specific scientific metric semantics.

## Contracts
- Every example run must produce a minimum output bundle with logs, metrics, traces, and report artifacts.
- Output paths must be deterministic relative to run root under
  [RUN_ARTIFACTS.md](../30-operations/RUN_ARTIFACTS.md).
- Machine-readable report outputs must satisfy [REPORT_CONTRACT.md](../30-operations/REPORT_CONTRACT.md).

## Required Outputs
- `logs/`:
  - command log
  - stderr/stdout capture or equivalent structured run log
- `metrics/`:
  - stage or run metrics payload(s)
  - summary metrics table/json
- `traces/`:
  - timeline/events trace (`events.jsonl` or equivalent)
  - provenance/trace identifiers
- `report_bundle/`:
  - human-readable report entrypoint (for example `index.html`)
  - machine-readable report JSON contract

## Verification Rules
- Missing any required category is a contract violation.
- Paths must resolve under the run output root.
