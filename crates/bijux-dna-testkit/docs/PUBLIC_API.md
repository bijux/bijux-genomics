# Public API

The crate root exports a stable test-helper surface from `src/lib.rs`.
`src/public_api/surface.rs` mirrors that surface for callers that prefer an
explicit namespace.

## Public Modules

- `determinism`
- `fixtures`
- `public_api`
- `snapshots`
- `temp`
- `workspace_support`

## Stable Root Exports

- `FixedClock`
- `fixed_rng`
- `assert_json_stable`
- `assert_stable_ordering`
- `strip_timestamp_fields`
- `assert_json_schema_like`
- `load_fixture_json`
- `load_fixture_text`
- `install_snapshot_env`
- `sanitize_snapshot_json`
- `sanitize_snapshot_text`
- `snapshot_name`
- `snapshot_normalize_json`
- `snapshot_normalize_text`
- `stable_json`
- `resolve_under`
- `sorted_read_dir_paths`
- `temp_path_for`
- `tempdir_for`
- `TestPaths`
- `read_policy_text`
- `workspace_root_from_manifest`

## Compatibility Rules

- Removing or renaming a stable root export is breaking.
- Changing snapshot normalization semantics is breaking unless covered by
  explicit snapshot updates and release notes.
- Adding a public helper requires `docs/COMMANDS.md`, this file, public API
  tests, and snapshot updates.
