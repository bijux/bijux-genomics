# Commands

This file is the SSOT for callable operations owned by
`bijux-dna-bench-model`.

## Managed Model Operations

These operations are pure Rust functions and type constructors; this crate does
not own CLI commands or process execution.

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `validate-suite` | `contract::validate_suite` | Validate `BenchmarkSuiteSpec` shape, graph, stage ids, tool ids, and parameter bindings. |
| `validate-observation` | `contract::validate_observation` | Validate one `BenchmarkObservation` before summarization. |
| `validate-summary` | `contract::validate_summary` | Validate a `BenchmarkSummary` before persistence or comparison. |
| `validate-decision` | `contract::validate_decision` | Validate a `GateDecision` before persistence. |
| `compare-summaries` | `compare::compare_summaries` | Build a deterministic typed comparison report from two summaries. |
| `gate-policy-decide` | `GatePolicy::decide` | Evaluate one summary row metric map against gate policy thresholds and overrides. |
| `robust-stats` | `stats::robust_stats` | Compute deterministic robust statistics. |
| `bootstrap-ci` | `stats::bootstrap_ci` | Compute seeded bootstrap confidence intervals. |
| `mad-outliers` | `stats::mad_outliers` | Detect deterministic median-absolute-deviation outliers. |

## Local Verification Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-bench-model --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test semantics --no-default-features
```
