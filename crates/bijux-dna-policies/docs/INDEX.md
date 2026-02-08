# bijux-dna-policies Docs Index

## Scope
Defines the policy suite that governs architecture, purity, and documentation placement.

## Entry points
- `ENFORCEMENT.md` — how to run policy gates.
- `TESTS.md` — mapping of tests to intent.
- `POLICY_MATRIX.md` — authoritative registry of policy tests.
- `POLICY_DIAGNOSTICS.md` — WHAT/WHY/HOW/MORE diagnostics format.
- `CHANGE_RULES.md` — breaking vs non-breaking policy changes.

## Effects
No runtime effects; policies only inspect source/layout.

## Extension Points
Add new policies in `tests/surface`, `tests/deps`, `tests/data`, or `tests/tooling` and document them in `TESTS.md` and `POLICY_MATRIX.md`.
Style policies must also be listed in `docs/40-policies/POLICY_MATRIX.md`.

## Boundaries
Policies enforce architectural boundaries across crates and ensure documentation and snapshots stay consistent.

## How to Test
Run `make guardrails`, `make policies`, and `make structure-check` as documented in `ENFORCEMENT.md`.

## Policy modules
Use the test file list in `TESTS.md` as the authoritative index.
