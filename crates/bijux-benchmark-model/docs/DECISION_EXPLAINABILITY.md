# DECISION_EXPLAINABILITY

## Explainability contract
Every decision must include:
- reasons: ordered list of human-readable rationale entries.
- weights: numeric weights per metric or factor.
- deltas: comparisons between candidates (winner vs baseline).

## Determinism
- reasons ordering must be stable.
- weights/deltas are deterministic for identical inputs.

## Enforcement
See `tests/semantics/decision_explainability.rs`.
