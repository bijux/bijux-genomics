#[test]
fn public_api_docs_list_modules_and_root_exports() {
    let public_api = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/PUBLIC_API.md"),
    )
    .unwrap_or_else(|err| panic!("read docs/PUBLIC_API.md: {err}"));

    for module in
        ["determinism", "fixtures", "public_api", "snapshots", "temp", "workspace_support"]
    {
        assert!(
            public_api.contains(&format!("`{module}`")),
            "docs/PUBLIC_API.md must list public module {module}"
        );
    }

    for export in [
        "FixedClock",
        "fixed_rng",
        "assert_json_stable",
        "assert_stable_ordering",
        "strip_timestamp_fields",
        "assert_json_schema_like",
        "load_fixture_json",
        "load_fixture_text",
        "install_snapshot_env",
        "sanitize_snapshot_json",
        "sanitize_snapshot_text",
        "snapshot_name",
        "snapshot_normalize_json",
        "snapshot_normalize_text",
        "stable_json",
        "resolve_under",
        "sorted_read_dir_paths",
        "temp_path_for",
        "tempdir_for",
        "TestPaths",
        "read_policy_text",
        "workspace_root_from_manifest",
    ] {
        assert!(
            public_api.contains(&format!("`{export}`")),
            "docs/PUBLIC_API.md must list root export {export}"
        );
    }
}
