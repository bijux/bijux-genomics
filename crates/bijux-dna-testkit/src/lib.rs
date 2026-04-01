//! Shared test fixtures/helpers for bijux crates.
//! This crate stays test-only, with one source file per support concern.

pub mod determinism;
pub mod fixtures;
pub mod public_api;
pub mod snapshots;
pub mod temp;
pub mod workspace_support;

pub use determinism::{
    assert_json_stable, assert_stable_ordering, fixed_rng, strip_timestamp_fields, FixedClock,
};
pub use fixtures::{assert_json_schema_like, load_fixture_json, load_fixture_text};
pub use snapshots::{
    install_snapshot_env, sanitize_snapshot_json, sanitize_snapshot_text, snapshot_name,
    snapshot_normalize_json, snapshot_normalize_text, stable_json,
};
pub use temp::{resolve_under, sorted_read_dir_paths, temp_path_for, tempdir_for, TestPaths};
pub use workspace_support::{read_text as read_policy_text, workspace_root_from_manifest};
