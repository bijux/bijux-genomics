# bijux-dna-bench-model Public API

The stable consumer surface is curated in `src/public_api/stable_surface.rs` and
then re-exported from `src/lib.rs`. Consumers should prefer the crate root for
stable names and use namespaces only when they need grouped operations such as
`contract::validate_suite` or `compare::compare_summaries`.

## Stable Root Exports

Model contracts:

- `BenchmarkSuiteSpec`
- `BenchmarkObservation`
- `BenchmarkSummary`
- `BenchmarkDecision`
- `DecisionRationale`
- `BenchmarkGraphNode`
- `BenchmarkGraphNodeKind`
- `BenchmarkStageEdge`
- `BenchmarkStageSpec`
- `BenchmarkParamBinding`
- `DatasetSpec`
- `AnalysisRequirements`
- `DiversityRequirements`
- `ReplicatePolicy`
- `StratificationRequirement`
- `MetricsEnvelope`
- `MetricSummary`
- `SummaryRow`
- `SummaryStratum`

Policy and diagnostics:

- `GatePolicy`
- `GatePolicyOverrides`
- `GateDecision`
- `GateViolation`
- `BenchError`

Statistics:

- `robust_stats`

Namespaces:

- `compare`
- `contract`

## Managed Operations

The callable pure-model operations are listed in `docs/COMMANDS.md`. Public API
docs must not invent a separate command list; the command document is the SSOT
for operation names and entrypoints.

## Compatibility Rules

- Additions must preserve serialization compatibility or include an explicit
  breaking-change review under `docs/CHANGE_RULES.md`.
- New public exports must be added through `src/public_api/stable_surface.rs` so
  schema tests can detect the change.
- Public validation behavior must be reflected in contract tests and the
  relevant docs.
- Public ordering of lists, reports, and rationale traces must remain
  deterministic for identical inputs.

## Verification

Run from the repository root when changing public exports:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test schemas --no-default-features
```
