# DOMAIN_MODEL

This crate owns FASTQ domain truth: IDs, params, metric semantics, and invariant verdicts.

## Domain truth set
- IDs: stage IDs and tool IDs live in `src/stages/ids.rs` and `src/id_catalog.rs`.
- Params: canonical JSON and defaults in `src/params/*` and `docs/PARAMS.md`.
- Metrics: meaning + ordering rules in `src/metrics/*` and `docs/METRICS.md`.
- Invariants: evaluation rules in `src/invariants/*` and `docs/FAILURE_PATTERNS.md`.

## Retention semantics
Retention is always scoped as `numerator/denominator` at a specific stage boundary.
Examples are enforced in `tests/semantics/retention_truth_table.rs` and
`tests/semantics/retention_semantics.rs`.

## Invariant verdicts
Verdicts are stable and mean:
- pass: invariant holds within thresholds.
- warn: invariant deviation detected; pipeline can continue but operator review required.
- fail: invariant violated; pipeline must treat as a hard contract failure.

Invariant expectations are enforced by the invariant suite:
- `tests/invariants/invariant_specs.rs`
- `tests/invariants/invariants.rs`

## Forbidden
- Tool selection
- Execution or runner concepts
