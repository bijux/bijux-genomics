# Change Rules

## Purpose
Prevent silent contract drift in analysis outputs, public API exports, report schemas, and
architecture boundaries.

## Non-Breaking Changes
- Adding an optional report field with absent-safe readers.
- Adding a new helper behind `src/public_api/` without changing existing names or semantics.
- Adding a new report section that is optional and documented in `docs/REPORT_CONTRACT.md`.
- Adding tests or fixtures that do not change blessed artifact output.

## Breaking Changes
- Removing or renaming public API items.
- Changing a report field type, meaning, requiredness, or stable section name.
- Changing ranking semantics, tie-breaks, or missing-data policy.
- Changing loaded artifact schema requirements.
- Moving ownership across `load`, `decision`, `report`, or `pipeline` in a way that weakens the
  dependency direction in `docs/ARCHITECTURE.md`.

Breaking changes require explicit approval, a schema or contract version decision, and reviewed
snapshot updates.

## Required Update Checklist
- Update `docs/REPORT_CONTRACT.md` for report or schema behavior.
- Update `docs/PUBLIC_API.md` for stable API changes.
- Update `docs/DECISIONS.md` for ranking, compare, trace, or missing-data semantics.
- Update `docs/COMMANDS.md` when crate-owned modes or package commands change.
- Update `docs/TESTS.md` when coverage moves or new suites become authoritative.
- Update fixtures and snapshots in `tests/fixtures/` and `tests/snapshots/` when blessed artifact
  output changes.

## Required Checks
Use `docs/COMMANDS.md` for command details. At minimum, run the suite that owns the changed
contract plus `tests/boundaries.rs` when source layout, docs layout, public surface, or dependency
direction changes.

## Failure Modes
- Unversioned breaking report changes fail contract or snapshot review.
- Public-surface creep fails boundary tests.
- Cross-layer imports fail guardrail tests.
- Undocumented docs growth fails docs-layout tests.
