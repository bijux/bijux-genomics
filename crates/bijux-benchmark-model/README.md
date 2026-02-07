# bijux-benchmark-model

## What this crate does
Pure statistical model for benchmark decisions.

## What it must not do (boundaries)
No I/O or hidden randomness.

## Role in the stack
Upstream: benchmark inputs. Downstream: benchmark decisions.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/MODEL_GLOSSARY.md`, `docs/STAT_ASSUMPTIONS.md`, `docs/GATE_POLICY.md`, `docs/DETERMINISM.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Decision structures only.

## Effects & determinism guarantees
Pure computation; determinism enforced by tests. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/decision_explainability.rs`, `tests/public_api.rs`, `tests/ssot_metrics.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
