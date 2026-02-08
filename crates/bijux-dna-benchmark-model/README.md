# bijux-dna-benchmark-model

## What this crate does
Pure statistical model for benchmark decisions.

## What it must not do (boundaries)
No I/O or hidden randomness.

## Role in the stack
Upstream: benchmark inputs. Downstream: benchmark decisions.

## Model philosophy
- Robust stats: minimize sensitivity to outliers.
- Outlier handling: detect and downweight extreme observations.
- Tie-breaking: deterministic and explainable (stable ordering rules).
- Explainability: every decision must provide reasons, weights, and deltas.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/MODEL_GLOSSARY.md`, `docs/STAT_ASSUMPTIONS.md`, `docs/GATE_POLICY.md`,
`docs/DETERMINISM.md`, `docs/COMPATIBILITY.md`, `docs/CHANGE_RULES.md`.

## Decision explainability contract
See `docs/DECISION_EXPLAINABILITY.md` and `tests/semantics/decision_explainability.rs`.

## Purity statement
Model functions are pure and deterministic; no RNG is allowed unless explicitly seeded and
recorded. This is enforced by `tests/guardrails/` and determinism tests in `tests/semantics/*`.

## Key contracts it owns/consumes
Public model types and their invariants:
- `Decision` (deterministic choice + rationale).
- `Suite` (collection of observations with stratification rules).
- `Observation` (single metric envelope with stable ids).
- `Summary` (aggregate outputs with ordering guarantees).

## Artifacts / Contracts
The model is pure code; its contract surface is defined by public types and snapshots.
See `tests/public_api/public_api.rs` and `tests/snapshots/public_api.txt`.

## Effects & determinism guarantees
Pure computation; determinism enforced by tests. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/semantics/decision_explainability.rs`,
`tests/public_api/public_api.rs`, `tests/semantics/ssot_metrics.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Public surface lock
The authoritative public surface snapshot lives at `tests/snapshots/public_api.txt`.
See `tests/public_api/public_api.rs`.

## Model glossary (authoritative)
All terms are defined in `docs/MODEL_GLOSSARY.md`. Do not redefine terms elsewhere.

## Start here in code
`src/lib.rs` → `src/model/*` → `src/compare/*`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
