# Change Rules

## What
Defines which policy changes are breaking vs non‑breaking and how to update snapshots.

## Why
Prevents silent weakening of governance rules.

## Non-goals
- Removing policies without explicit review.

## Contracts
Breaking changes:
- Removing a policy or weakening enforcement.
- Changing required docs placement or naming.
- Renaming policy files without updating the index and snapshots.

Non‑breaking changes:
- Adding new policy tests with documentation and snapshots.
- Extending error messages with additional guidance.

## Examples
- Adding `docs_links.rs` requires adding it to INDEX.md.

## Failure modes
- Missing snapshot updates cause policy failures.

## Snapshot bumps
Snapshot updates must accompany structural doc changes. Treat snapshot changes as contract changes and review them explicitly.

## Checklist
- [ ] Update `docs/INDEX.md` with any new policy test file.
- [ ] Update snapshots (`docs_tree_contract.snap`, `crate_docs_tree_contract.snap`) if docs layout changes.
- [ ] Ensure policy diagnostics follow the standard format.
