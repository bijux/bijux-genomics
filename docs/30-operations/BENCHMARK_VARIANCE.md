# Benchmark Variance Policy

## Purpose
Define acceptable run-to-run variance for frontend mini benchmark integrity checks.

## Scope
Mini benchmark integrity runs executed via `scripts/run.sh tooling benchmark-integrity-mini`.

## Non-goals
This policy does not define domain-science correctness thresholds.

## Contracts
- Runtime relative variance threshold: `<= 0.20` (20%).
- Memory relative variance threshold: `<= 0.25` (25%).
- `report.html` normalized structure must match across consecutive runs.
- Metrics numeric precision must be stable (no excessive floating-point noise).

Source of truth:
- `configs/bench/knobs.toml` `[variance]` section.

Failure modes:
- If thresholds are exceeded, treat as non-deterministic run conditions and investigate host load, container/image drift, or input changes.
