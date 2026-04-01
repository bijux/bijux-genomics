# bijux-dna-policies

## What this crate does
Enforces workspace policies for source layout, dependency boundaries, documentation hygiene, and deterministic governance checks.

## What it must not do (boundaries)
No product execution, domain semantics, or runtime orchestration; this crate only inspects source, docs, and fixtures to emit policy diagnostics.

## How to run policies
- `make guardrails`
- `make policies`
- `make structure-check`

## How to interpret failures
- Messages follow the WHAT/WHY/HOW/MORE format (see `crates/bijux-dna-policies/docs/POLICY_DIAGNOSTICS.md`).
- Use the failing test path to locate the rule family and the owning directory in `src/`.
- Re-run the exact test (or the full enforcement workflow) after fixing the violation.

## How to add a policy
- Add or extend a test under the matching intent directory in `crates/bijux-dna-policies/tests/`.
- If the policy needs reusable execution support, place it under `src/checks/`, `src/guardrails/`, `src/source_scan/`, or `src/policy_diagnostics/` instead of expanding `lib.rs`.
- Document it in `crates/bijux-dna-policies/docs/TESTS.md` and `crates/bijux-dna-policies/docs/POLICY_MATRIX.md`.
- Add or update snapshots only when the change is intentional (see `crates/bijux-dna-policies/docs/BLESS_WORKFLOW.md`).

## Effects & determinism guarantees
Pure filesystem inspection and snapshot checks only. Source scanning is deterministic, and diagnostics are rendered through a single WHAT/WHY/HOW/MORE contract.

## Public API / entrypoints
- Stable root exports: `check`, `GuardrailConfig`, `policy_assert!`, `policy_assert_eq!`, `policy_assert_ne!`, `policy_panic!`.
- Public modules: `public_api` and `policy_diagnostics`.
- Internal layout: `guardrails/` owns configuration, presets, and runner wiring; `checks/` owns rule families; `source_scan/` owns Rust file discovery.

## Key contracts it owns/consumes
Owns the guardrail configuration schema, policy diagnostic format, and the source-tree contract for policy execution support.

## Artifacts / Contracts
See `crates/bijux-dna-policies/docs/POLICY_MATRIX.md`, `crates/bijux-dna-policies/docs/POLICY_DIAGNOSTICS.md`, and `crates/bijux-dna-policies/docs/ARCHITECTURE.md`.

## Docs entrypoints
See `crates/bijux-dna-policies/docs/INDEX.md`, `crates/bijux-dna-policies/docs/ENFORCEMENT.md`, `crates/bijux-dna-policies/docs/POLICY_DIAGNOSTICS.md`, and `crates/bijux-dna-policies/docs/TESTS.md`.

## Failure modes
Policy failures report WHAT/WHY/HOW/MORE, usually from the shared assertion macros, and point back to the owning policy family.

## How to run its tests
See `crates/bijux-dna-policies/docs/TESTS.md`. Stable entrypoints are `tests/boundaries.rs`, `tests/contracts.rs`, `tests/determinism.rs`, and `tests/guardrails.rs`.

## Where the docs live
Start at `crates/bijux-dna-policies/docs/INDEX.md`, then use `ARCHITECTURE.md`, `TESTS.md`, `POLICY_MATRIX.md`, and `POLICY_DIAGNOSTICS.md` for the source tree, suite map, rule index, and failure format.

## Stability
Contract and behavior changes follow `crates/bijux-dna-policies/docs/CHANGE_RULES.md`.
