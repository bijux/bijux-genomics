use anyhow::Result;
use bijux_dna_bench_model::{
    validate_governed_data_catalog_registry, GovernedDataCatalogRegistry,
    GovernedDataCatalogRegistryRecord, GovernedDataCatalogSnapshot,
    GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SCHEMA_V2, GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1,
};

#[test]
fn registry_retains_history_and_selects_current_v2_snapshot() {
    assert!(validate_governed_data_catalog_registry(&registry()).is_ok());
}

#[test]
fn registry_rejects_legacy_current_and_identity_drift() {
    let mut value = registry();
    value.records[0].current_catalog_sha256 = "a".repeat(64);
    assert!(validate_governed_data_catalog_registry(&value)
        .is_err_and(|error| error.to_string().contains("not a native publication schema")));

    let mut value = registry();
    value.records[0].snapshots[1].catalog_path =
        "study://data/catalog/human-short-read-diversity.json".to_string();
    assert!(validate_governed_data_catalog_registry(&value)
        .is_err_and(|error| error.to_string().contains("source addressed")));

    let mut value = registry();
    value.records[0].snapshots[1].catalog_sha256 = "a".repeat(64);
    assert!(validate_governed_data_catalog_registry(&value)
        .is_err_and(|error| error.to_string().contains("repeats a snapshot")));
}

#[test]
fn registry_accepts_a_current_derived_catalog_snapshot() {
    let mut value = registry();
    value.records[0].snapshots[1].catalog_schema_version =
        GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1.to_string();
    assert!(validate_governed_data_catalog_registry(&value).is_ok());
}

#[test]
fn checked_registry_schema_matches_model_and_rejects_unknown_fields() -> Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/governed-data-catalog-registry.schema.json");
    let generated =
        serde_json::to_value(bijux_dna_bench_model::governed_data_catalog_registry_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated);

    let mut persisted = serde_json::to_value(registry())?;
    persisted
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("registry is not an object"))?
        .insert("unowned".to_string(), true.into());
    assert!(serde_json::from_value::<GovernedDataCatalogRegistry>(persisted).is_err());
    Ok(())
}

fn registry() -> GovernedDataCatalogRegistry {
    GovernedDataCatalogRegistry {
        schema_version: GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1.to_string(),
        created_at_utc: "2026-08-08T00:00:00Z".to_string(),
        data_root: "study://data".to_string(),
        records: vec![GovernedDataCatalogRegistryRecord {
            id: "human-short-read-diversity".to_string(),
            current_catalog_sha256: "b".repeat(64),
            snapshots: vec![
                GovernedDataCatalogSnapshot {
                    catalog_path: format!(
                        "study://data/catalog/snapshots/{}/human-short-read-diversity.json",
                        "1".repeat(40)
                    ),
                    catalog_sha256: "a".repeat(64),
                    catalog_schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V1.to_string(),
                    source_commit: "1".repeat(40),
                    created_at_utc: "2026-08-07T00:00:00Z".to_string(),
                },
                GovernedDataCatalogSnapshot {
                    catalog_path: format!(
                        "study://data/catalog/snapshots/{}/human-short-read-diversity.json",
                        "2".repeat(40)
                    ),
                    catalog_sha256: "b".repeat(64),
                    catalog_schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V2.to_string(),
                    source_commit: "2".repeat(40),
                    created_at_utc: "2026-08-08T00:00:00Z".to_string(),
                },
            ],
        }],
    }
}
