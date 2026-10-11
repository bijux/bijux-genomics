use std::path::PathBuf;

use bijux_dna_bench_model::{
    validate_benchmark_campaign_report_publication,
    validate_governed_data_catalog_registry_publication,
    validate_governed_data_catalog_snapshot_publication, BenchmarkCampaignReportPublication,
    GovernedDataCatalogRegistry, GovernedDataCatalogRegistryPublication,
    GovernedDataCatalogRegistryRecord, GovernedDataCatalogSnapshot,
    GovernedDataCatalogSnapshotPublication, BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_REGISTRY_PUBLICATION_SCHEMA_V1, GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SCHEMA_V2, GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1,
};

#[test]
fn publication_outcomes_validate_complete_immutable_identities() {
    assert!(validate_governed_data_catalog_snapshot_publication(&snapshot_publication()).is_ok());
    assert!(validate_governed_data_catalog_registry_publication(&registry_publication()).is_ok());
    assert!(validate_benchmark_campaign_report_publication(&report_publication()).is_ok());
}

#[test]
fn publication_outcomes_reject_count_digest_and_overlay_drift() {
    let mut snapshot = snapshot_publication();
    snapshot.retained_snapshot_count = 1;
    assert!(validate_governed_data_catalog_snapshot_publication(&snapshot)
        .is_err_and(|error| error.to_string().contains("counts drifted")));

    let mut registry = registry_publication();
    registry.registry_sha256 = "0".repeat(64);
    assert!(validate_governed_data_catalog_registry_publication(&registry)
        .is_err_and(|error| error.to_string().contains("identity is invalid")));

    let mut report = report_publication();
    report.presentation_overlaid_stage_ids.push("bam.coverage".to_string());
    assert!(validate_benchmark_campaign_report_publication(&report)
        .is_err_and(|error| error.to_string().contains("overlays are invalid")));
}

#[test]
fn publication_outcome_schemas_match_model_and_versions_fail_closed() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    for (name, generated) in [
        (
            "governed-data-catalog-snapshot-publication.schema.json",
            serde_json::to_value(
                bijux_dna_bench_model::governed_data_catalog_snapshot_publication_schema(),
            )?,
        ),
        (
            "governed-data-catalog-registry-publication.schema.json",
            serde_json::to_value(
                bijux_dna_bench_model::governed_data_catalog_registry_publication_schema(),
            )?,
        ),
        (
            "campaign-report-publication.schema.json",
            serde_json::to_value(
                bijux_dna_bench_model::benchmark_campaign_report_publication_schema(),
            )?,
        ),
    ] {
        let path = repository_root.join("benchmarks/contracts/schemas").join(name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "publication outcome schema drifted: {name}");
    }

    let mut value = serde_json::to_value(report_publication())?;
    value["schema_version"] = serde_json::json!("bijux.benchmark.campaign_report_publication.v2");
    assert!(serde_json::from_value::<BenchmarkCampaignReportPublication>(value).is_err());
    Ok(())
}

fn snapshot_publication() -> GovernedDataCatalogSnapshotPublication {
    GovernedDataCatalogSnapshotPublication {
        schema_version: GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1.to_string(),
        candidate_registry_path: PathBuf::from("/study/data/evidence/catalog/registry.json"),
        preserved_snapshot_count: 1,
        retained_snapshot_count: 0,
        registry: registry(),
    }
}

fn registry_publication() -> GovernedDataCatalogRegistryPublication {
    GovernedDataCatalogRegistryPublication {
        schema_version: GOVERNED_DATA_CATALOG_REGISTRY_PUBLICATION_SCHEMA_V1.to_string(),
        registry_path: PathBuf::from(format!(
            "/study/data/catalog/registries/{}.json",
            "1".repeat(40)
        )),
        registry_sha256: "a".repeat(64),
        registry: registry(),
    }
}

fn report_publication() -> BenchmarkCampaignReportPublication {
    BenchmarkCampaignReportPublication {
        schema_version: BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1.to_string(),
        run_id: "run-2002".to_string(),
        stage_id: "bam.coverage".to_string(),
        report_path: PathBuf::from("/study/benchmarks/reports/run-2002.json"),
        report_sha256: "b".repeat(64),
        markdown_path: PathBuf::from("/study/benchmarks/reports/run-2002.md"),
        markdown_sha256: "c".repeat(64),
        presentation_repository_commit: "1".repeat(40),
        presentation_catalog_sha256: "d".repeat(64),
        presentation_overlaid_stage_ids: vec!["bam.coverage".to_string()],
    }
}

fn registry() -> GovernedDataCatalogRegistry {
    GovernedDataCatalogRegistry {
        schema_version: GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1.to_string(),
        created_at_utc: "2026-08-10T00:00:00Z".to_string(),
        data_root: "study://data".to_string(),
        records: vec![GovernedDataCatalogRegistryRecord {
            id: "human-short-read-diversity".to_string(),
            current_catalog_sha256: "e".repeat(64),
            snapshots: vec![GovernedDataCatalogSnapshot {
                catalog_path: format!(
                    "study://data/catalog/snapshots/{}/human-short-read-diversity.json",
                    "1".repeat(40)
                ),
                catalog_sha256: "e".repeat(64),
                catalog_schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V2.to_string(),
                source_commit: "1".repeat(40),
                created_at_utc: "2026-08-10T00:00:00Z".to_string(),
            }],
        }],
    }
}
