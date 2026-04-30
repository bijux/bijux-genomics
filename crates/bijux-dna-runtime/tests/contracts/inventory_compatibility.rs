use std::path::Path;

use bijux_dna_core::contract::ManifestMigrationStatusV1;
use bijux_dna_runtime::run_layout::{migrate_artifact_inventory_value, read_supported_artifact_inventory};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/runtime_schema/default").join(name)
}

#[test]
fn legacy_artifact_inventory_is_upgraded_with_audit() -> anyhow::Result<()> {
    let raw = std::fs::read_to_string(fixture("artifact_inventory_v0.json"))?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let (inventory, audit) = migrate_artifact_inventory_value(&value)?;

    assert_eq!(inventory.schema_version, "bijux.artifact_inventory.v1");
    assert_eq!(inventory.artifacts.len(), 1);
    assert_eq!(audit.status, ManifestMigrationStatusV1::Upgraded);
    assert_eq!(
        audit.exact_reason,
        "artifact inventory upgraded from governed legacy v0 by materializing explicit replay and scientific context fields"
    );
    Ok(())
}

#[test]
fn artifact_inventory_reader_accepts_supported_legacy_fixture() -> anyhow::Result<()> {
    let (inventory, audit) = read_supported_artifact_inventory(&fixture("artifact_inventory_v0.json"))?;
    assert_eq!(inventory.run_id, "run-1");
    assert_eq!(audit.from_schema_version, "bijux.artifact_inventory.v0");
    Ok(())
}

#[test]
fn unsupported_artifact_inventory_versions_are_refused_cleanly() {
    let value = serde_json::json!({
        "schema_version": "bijux.artifact_inventory.v9",
        "run_id": "run-1",
        "artifacts": []
    });
    let err = migrate_artifact_inventory_value(&value).unwrap_err();
    assert_eq!(
        err.to_string(),
        "validation error: artifact_inventory schema_version bijux.artifact_inventory.v9 is unsupported; supported versions: bijux.artifact_inventory.v0, bijux.artifact_inventory.v1"
    );
}
