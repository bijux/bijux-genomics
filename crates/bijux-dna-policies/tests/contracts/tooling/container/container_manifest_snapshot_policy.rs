#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

fn normalized_json(input: &str) -> String {
    let value: serde_json::Value =
        serde_json::from_str(input).unwrap_or_else(|err| panic!("parse manifest json: {err}"));
    serde_json::to_string_pretty(&value)
        .unwrap_or_else(|err| panic!("pretty-print manifest json: {err}"))
}

#[test]
fn slow__policy__contracts__container_manifest_snapshot_policy__generated_manifest_matches_committed_snapshot(
) {
    let root = support::workspace_root();
    let registry_path = root.join("configs/ci/registry/tool_registry.toml");
    let snapshot_path = root.join(
        "crates/bijux-dna-policies/tests/fixtures/container_manifest_snapshot/manifest.snapshot.json",
    );
    let expected = std::fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", snapshot_path.display()));
    let actual = serde_json::to_string_pretty(
        &bijux_dna::public_api::cli::env::registry_export_containers_value(&registry_path)
            .unwrap_or_else(|err| {
                panic!("render container manifest with {}: {err}", registry_path.display())
            }),
    )
    .expect("manifest output utf8");

    assert_eq!(
        normalized_json(&actual).trim(),
        normalized_json(&expected).trim(),
        "container manifest snapshot is stale. Regenerate with: bijux-dna registry export-containers --json > crates/bijux-dna-policies/tests/fixtures/container_manifest_snapshot/manifest.snapshot.json"
    );
}
