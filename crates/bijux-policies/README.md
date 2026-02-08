# bijux-policies

## What this crate is
Policy enforcement for boundaries, ownership, and documentation across the workspace.

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

## Boundaries
No product logic; only diagnostics and policy assertions.

## Docs entrypoints
See `docs/INDEX.md`, `docs/ENFORCEMENT.md`, `docs/POLICY_DIAGNOSTICS.md`, and `docs/TESTS.md`.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
