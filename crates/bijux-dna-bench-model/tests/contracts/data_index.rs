use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use bijux_dna_bench_model::{
    validate_benchmark_data_index, BenchmarkDataIndex, BenchmarkDataIndexFile,
    BenchmarkDataIndexRecord, BenchmarkDataIndexSummary, BenchmarkEligibility,
    BenchmarkTemporalProvenance, DataIntegrityStatus, BENCHMARK_DATA_INDEX_SCHEMA_V1,
};

#[test]
fn governed_data_index_accepts_complete_closure() {
    assert!(validate_benchmark_data_index(&valid_index()).is_ok());
}

#[test]
fn governed_data_index_rejects_summary_and_provenance_drift() {
    let mut index = valid_index();
    index.summary.file_count = 2;
    assert!(validate_benchmark_data_index(&index)
        .is_err_and(|error| error.to_string().contains("summary")));

    let mut index = valid_index();
    index.records[0].source_identities.clear();
    assert!(validate_benchmark_data_index(&index)
        .is_err_and(|error| error.to_string().contains("missing provenance")));
}

#[test]
fn governed_data_index_rejects_unknown_persisted_fields() -> Result<()> {
    let mut value = serde_json::to_value(valid_index())?;
    value
        .as_object_mut()
        .ok_or_else(|| anyhow!("serialized data index is not an object"))?
        .insert("unowned".to_string(), true.into());
    assert!(serde_json::from_value::<BenchmarkDataIndex>(value).is_err());
    Ok(())
}

#[test]
fn governed_data_index_v2_requires_exact_registry_identity() {
    let mut index = valid_index();
    index.schema_version = bijux_dna_bench_model::BENCHMARK_DATA_INDEX_SCHEMA_V2.to_string();
    index.catalog_registry_path = Some("study://data/catalog/registry.json".to_string());
    index.catalog_registry_sha256 = Some("c".repeat(64));
    assert!(validate_benchmark_data_index(&index).is_ok());

    index.catalog_registry_sha256 = None;
    assert!(validate_benchmark_data_index(&index)
        .is_err_and(|error| error.to_string().contains("disagrees with schema")));
}

fn valid_index() -> BenchmarkDataIndex {
    let file = BenchmarkDataIndexFile {
        path: "study://data/fastq/ERR000001/ERR000001.fastq.gz".to_string(),
        sha256: "a".repeat(64),
        size_bytes: 42,
        role: "raw-fastq".to_string(),
        legacy_fields: vec!["path".to_string(), "sha256".to_string(), "size_bytes".to_string()],
    };
    let record = BenchmarkDataIndexRecord {
        id: "fixture-fastq".to_string(),
        catalog_path: "study://data/catalog/fixture-fastq.json".to_string(),
        catalog_sha256: "b".repeat(64),
        purpose: "benchmark fixture".to_string(),
        source_identities: vec!["source:https://example.invalid/fixture".to_string()],
        access_classification: "public".to_string(),
        redistribution_classification: "permitted".to_string(),
        scientific_name: Some("Example species".to_string()),
        reference_assembly: None,
        temporal_provenance: BenchmarkTemporalProvenance {
            field: Some("retrieved_at_utc".to_string()),
            timestamp: Some("2026-08-06T00:00:00+00:00".to_string()),
        },
        files: vec![file],
        known_limitations: vec!["synthetic fixture".to_string()],
        integrity_status: DataIntegrityStatus::Verified,
        benchmark_eligibility: BenchmarkEligibility::Eligible,
        missing_provenance: Vec::new(),
    };
    BenchmarkDataIndex {
        schema_version: BENCHMARK_DATA_INDEX_SCHEMA_V1.to_string(),
        created_at_utc: "2026-08-06T00:00:00+00:00".to_string(),
        data_root: "study://data".to_string(),
        catalog_registry_path: None,
        catalog_registry_sha256: None,
        summary: BenchmarkDataIndexSummary {
            record_count: 1,
            file_count: 1,
            size_bytes: 42,
            integrity_verified_count: 1,
            benchmark_eligibility: BTreeMap::from([(BenchmarkEligibility::Eligible, 1)]),
        },
        records: vec![record],
    }
}
