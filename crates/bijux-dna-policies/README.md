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
- Messages follow the WHAT/WHY/HOW/MORE format (see `crates/bijux-dna-policies/docs/POLICY_DIAGNOSTICS.md`).
- Use the test path in the failure to locate the rule and update the doc listed in `crates/bijux-dna-policies/docs/TESTS.md`.
- Re-run the exact test (or the full enforcement workflow) after fixing the violation.

## How to add a policy
- Add a new test under `crates/bijux-dna-policies/tests/`.
- Document it in `crates/bijux-dna-policies/docs/TESTS.md` and `crates/bijux-dna-policies/docs/POLICY_MATRIX.md`.
- Add or update snapshots only when the change is intentional (see `crates/bijux-dna-policies/docs/BLESS_WORKFLOW.md`).
- If it is a style rule, add it to `crates/bijux-dna-policies/docs/POLICY_MATRIX.md`.

## Effects & determinism guarantees
Pure filesystem inspection and snapshot checks; deterministic outputs only.

## Public API / entrypoints
See `crates/bijux-dna-policies/docs/INDEX.md` and the policy index in `crates/bijux-dna-policies/docs/POLICY_MATRIX.md`.

## Key contracts it owns/consumes
Owns policy diagnostics format and enforcement matrix.

## Artifacts / Contracts
See `crates/bijux-dna-policies/docs/POLICY_MATRIX.md` and `crates/bijux-dna-policies/docs/POLICY_DIAGNOSTICS.md`.

## Docs entrypoints
See `crates/bijux-dna-policies/docs/INDEX.md`, `crates/bijux-dna-policies/docs/ENFORCEMENT.md`, `crates/bijux-dna-policies/docs/POLICY_DIAGNOSTICS.md`, and `crates/bijux-dna-policies/docs/TESTS.md`.

## Failure modes
Policy failures report WHAT/WHY/HOW/MORE with links to docs.

## How to run its tests
See `crates/bijux-dna-policies/docs/TESTS.md`. Policies are enforced via `make guardrails`, `make policies`, `make structure-check`.
Key tests: `tests/boundaries.rs`, `tests/contracts.rs`, `tests/guardrails.rs`, `tests/determinism.rs`.

## Where the docs live
Start at `crates/bijux-dna-policies/docs/INDEX.md` and follow the policy docs listed above.

## Stability
Contract and behavior changes follow `crates/bijux-dna-policies/docs/CHANGE_RULES.md`.
