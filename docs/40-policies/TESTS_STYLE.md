# Tests Style Guide

## What
Defines the workspace-wide conventions for tests, fixtures, and snapshots.

## Why
Consistent structure keeps policies enforceable and tests easy to navigate.

## Non-goals
- Replacing per-crate test rationale docs.
- Documenting every test case.

## Contracts
- Policy enforcement lives under
  [crates/bijux-dna-policies/tests/contracts/](../../crates/bijux-dna-policies/tests/contracts/).
- Shared test helpers and workspace fixtures live under
  [crates/bijux-dna-policies/tests/support/](../../crates/bijux-dna-policies/tests/support/).
- Snapshot sanitization helpers and shared testkit utilities live under
  [crates/bijux-dna-testkit/README.md](../../crates/bijux-dna-testkit/README.md).

## Examples
See the sections below for the required layout and naming rules enforced through those
surfaces.

## Failure modes
- Divergent layouts and naming cause policy failures and test drift.

## Canonical buckets
All crate tests live under `tests/` in one of:
- `contracts`
- `schemas`
- `semantics`
- `determinism`
- `boundaries`
- `fixtures`
- `snapshots`

Any other bucket must be justified by a `README.md` in that directory.

## Naming
- Test filenames must describe the rule under test.
- Avoid generic `helpers.rs` or `support.rs` unless required and documented.

## Fixtures
- Layout: `tests/fixtures/<domain_or_feature>/<case_name>/...`
- Every fixture directory must include `CASE.toml` or `CASE.json` explaining intent and invariants.
- No fixture file > 200KB unless allowlisted.

## Snapshots
- Snapshot names must follow: `<crate>__<bucket>__<test_name>.snap`.
- Tests must document what a snapshot proves and allowable changes.
- Debug snapshots are not committed; keep under `target/` or ignored paths.
- New snapshot tests must include a short rationale comment and an "Allowed changes" note in the test body or module docs.
- Use `bijux_dna_testkit::sanitize_snapshot_json` / `sanitize_snapshot_text` to strip unstable fields (paths, temp dirs) before snapshotting.

## Determinism
- Deterministic JSON serialization must be used for snapshots.
- Tests should strip permitted timestamps and compare stable outputs.

## Network
- Tests must not access the network.
