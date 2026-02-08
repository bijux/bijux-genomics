# bijux-dna-policies

## What this crate does
Policy enforcement for boundaries, ownership, and documentation across the workspace.

## What it must not do (boundaries)
No product logic, execution, or domain semantics; policies are diagnostics only.

## How to run policies
- `make guardrails`
- `make policies`
- `make structure-check`

## How to interpret failures
- Messages follow the WHAT/WHY/HOW/MORE format (see `docs/POLICY_DIAGNOSTICS.md`).
- Use the test path in the failure to locate the rule and update the doc listed in `docs/TESTS.md`.
- Re-run the exact test (or the full enforcement workflow) after fixing the violation.

## How to add a policy
- Add a new test under `crates/bijux-policies/tests/...`.
- Document it in `docs/TESTS.md` and `docs/POLICY_MATRIX.md`.
- Add or update snapshots only when the change is intentional (see `docs/BLESS_WORKFLOW.md`).
- If it is a style rule, add it to `docs/40-policies/POLICY_MATRIX.md`.

## Effects & determinism guarantees
Pure filesystem inspection and snapshot checks; deterministic outputs only.

## Public API / entrypoints
See `docs/INDEX.md` and the policy index in `docs/POLICY_MATRIX.md`.

## Key contracts it owns/consumes
Owns policy diagnostics format and enforcement matrix.

## Artifacts / Contracts
See `docs/POLICY_MATRIX.md` and `docs/POLICY_DIAGNOSTICS.md`.

## Docs entrypoints
See `docs/INDEX.md`, `docs/ENFORCEMENT.md`, `docs/POLICY_DIAGNOSTICS.md`, and `docs/TESTS.md`.

## Failure modes
Policy failures report WHAT/WHY/HOW/MORE with links to docs.

## How to run its tests
See `docs/TESTS.md`. Policies are enforced via `make guardrails`, `make policies`, `make structure-check`.
Key tests: `tests/boundaries.rs`, `tests/contracts.rs`, `tests/guardrails.rs`, `tests/determinism.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the policy docs listed above.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
