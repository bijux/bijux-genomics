use anyhow::Context as _;

fn normalized_json(input: &str) -> String {
    let value: serde_json::Value =
        serde_json::from_str(input).unwrap_or_else(|err| panic!("parse manifest json: {err}"));
    serde_json::to_string_pretty(&value)
        .unwrap_or_else(|err| panic!("pretty-print manifest json: {err}"))
}

#[test]
fn registry_export_matches_committed_manifest_snapshot() -> anyhow::Result<()> {
    let root = bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
    let registry_path = root.join("configs/ci/registry/tool_registry.toml");
    let snapshot_path = root
        .join("crates/bijux-dna-containers/tests/fixtures/registry_export/manifest.snapshot.json");
    let expected = std::fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", snapshot_path.display()));
    let actual = serde_json::to_string_pretty(
        &bijux_dna_containers::api::registry_export_containers_value(&registry_path)
            .unwrap_or_else(|err| {
                panic!("render container manifest with {}: {err}", registry_path.display())
            }),
    )
    .context("manifest output utf8")?;

    assert_eq!(
        normalized_json(&actual).trim(),
        normalized_json(&expected).trim(),
        "container manifest snapshot is stale. Regenerate with: bijux-dna registry export-containers --json > crates/bijux-dna-containers/tests/fixtures/registry_export/manifest.snapshot.json"
    );
    Ok(())
}
