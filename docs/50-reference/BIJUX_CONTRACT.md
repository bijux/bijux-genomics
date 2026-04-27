# Bijux Contract

## What
High‑level contract describing platform expectations.

## Why
Sets baseline behavior for all pipelines.

## Non-goals
- Tool‑specific contracts.

## Contracts
- RunManifest must exist for every run under
  [RUN_ARTIFACTS.md](../30-operations/RUN_ARTIFACTS.md).
- Replay and baseline platform guarantees are constrained by
  [REPRODUCIBILITY.md](../30-operations/REPRODUCIBILITY.md).

## Examples
- `run_manifest.json` is mandatory.

## Failure modes
- Missing manifest fails contract enforcement.
