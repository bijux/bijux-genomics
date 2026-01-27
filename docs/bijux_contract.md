# Bijux Contract (v1)

This document defines what Bijux **guarantees**, and what it **does not** guarantee.

## Guarantees

- **Deterministic execution manifests** for every run.
- **Frozen tool contracts** per stage (inputs/outputs/side-effects).
- **Typed metric schemas** per stage (no ad-hoc metrics).
- **Invariant enforcement** (invalid metrics hard-fail).
- **Comparable execution metrics** (runtime, memory, exit code) measured consistently.
- **Image QA gating** before benchmarks.
- **Reproducible runs** via `bijux replay <run-id>`.

## Non-guarantees

- No guarantees about biological correctness beyond defined proxies.
- No guarantees that tools are optimal for every dataset.
- No hidden tuning or parameter sweeps.
- No auto-selection unless explicitly requested.

## Metric measurement

- runtime_s: wall-clock time from container start to exit.
- memory_mb: container memory usage sampled with `docker stats --no-stream` after exit.

## Comparison policy

- Tools are compared only on schema-defined metrics.
- Derived metrics are computed deterministically from measured metrics.
- Sanity flags are advisory; they do not change outcomes.

## Stability

- Stage contracts and metric schemas are frozen for v1.
- Changes require a version bump and explicit documentation.
