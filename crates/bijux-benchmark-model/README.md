# bijux-benchmark-model

## What this crate does
Pure statistical model for benchmark decisions; deterministic and effect-free.

## What it must not do (boundaries)
Must not perform I/O, spawn processes, or depend on execution crates.

## Public API / entrypoints
Model contracts documented in `docs/MODEL_GLOSSARY.md` and `docs/GATE_POLICY.md`.

## Key contracts it owns/consumes
Owns statistical decision logic; consumes metric definitions from contracts.

## Effects & determinism guarantees
No effects; determinism and seeding rules enforced by tests. See `docs/DETERMINISM.md`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/decision_explainability.rs`, `tests/public_api.rs`, `tests/ssot_metrics.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/STAT_ASSUMPTIONS.md`, `docs/GATE_POLICY.md`, and `docs/DETERMINISM.md`.

## Artifacts / Contracts
Produces decision structures used by benchmark and analyze; no runtime artifacts.

## Failure modes
Invariants and explainability violations fail model tests.

## Stability
Public surface is snapshot-tested; see `docs/CHANGE_RULES.md`.
