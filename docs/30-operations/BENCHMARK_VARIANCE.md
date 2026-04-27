# Benchmark Variance Policy

## Purpose
Define acceptable run-to-run variance for frontend mini benchmark integrity checks.

## Scope
Mini benchmark integrity runs executed via `cargo run -q -p bijux-dna-dev -- tooling run benchmark-integrity-mini`.

This helper is not a separate benchmark control plane. It runs two consecutive `bijux-dna bench fastq validate` executions against the same governed input and compares the emitted artifacts for drift.

## Non-goals
This policy does not define domain-science correctness thresholds.

## Contracts
- Runtime relative variance threshold: `<= 0.20` (20%).
- Memory relative variance threshold: `<= 0.25` (25%).
- `report.html` normalized structure must match across consecutive runs.
- Metrics numeric precision must be stable (no excessive floating-point noise).

Source of truth:
- [configs/bench/knobs.toml](../../configs/bench/knobs.toml) `[variance]` section.

Failure modes:
- If thresholds are exceeded, treat as non-deterministic run conditions and investigate host load, container/image drift, or input changes.
