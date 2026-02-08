# Policy Diagnostics

## What
Standardizes policy failure messages so they are actionable and consistent.

## Why
Consistent diagnostics reduce triage time and prevent ambiguity during enforcement.

## Contracts
Each policy failure must include:
- **WHAT** — what failed.
- **WHY** — why the rule matters.
- **HOW** — how to fix it.
- **MORE** — where to read more.

## Example
WHAT: `crates/bijux-dna-core/docs/SCOPE.md` missing
WHY: Policies protect architectural boundaries, ownership, and determinism across the workspace.
HOW: Add `crates/bijux-dna-core/docs/SCOPE.md` with the required sections.
MORE: `crates/bijux-dna-policies/docs/TESTS.md`

## Failure modes
- Policy output omits any of the required sections.
