# Change Rules

## What
Defines breaking vs non‑breaking changes for `bijux-dna-analyze`.

## Why
Prevents silent contract drift.

## Non-goals
- Automatic versioning.

## Contracts
- Breaking changes require explicit approval and snapshot updates.

## Examples
- Changing a public contract field is breaking.

## Failure modes
- Unversioned breaking changes are rejected in CI.

## Schema change checklist
- Update `docs/REPORT_CONTRACT.md` and bump schema version if required.
- Update golden fixtures and snapshots in `tests/fixtures/` and `tests/snapshots/`.
- Update `docs/TESTS.md` to reflect the new coverage.
- Run `tests/report/report_contract.rs` and `tests/report/report_determinism.rs`.
