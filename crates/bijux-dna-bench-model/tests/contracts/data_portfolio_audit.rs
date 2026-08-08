use anyhow::Context;
use bijux_dna_bench_model::{
    benchmark_data_portfolio_audit_schema, validate_benchmark_data_portfolio_audit,
    BenchmarkCorpusClass, BenchmarkCorpusDataAudit, BenchmarkCorpusStatus,
    BenchmarkDataPortfolioAudit, BenchmarkDataPortfolioAuditSummary, BenchmarkEvidenceScope,
    BenchmarkGovernedDataBinding, BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2,
};
use std::path::{Path, PathBuf};

fn audit() -> BenchmarkDataPortfolioAudit {
    BenchmarkDataPortfolioAudit {
        schema_version: BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2.to_string(),
        source_commit: "1".repeat(40),
        portfolio_path: "benchmarks/contracts/portfolio.toml".to_string(),
        portfolio_sha256: "2".repeat(64),
        data_index_path: "study://data/evidence/indexes/source/data-index.json".to_string(),
        data_index_sha256: "3".repeat(64),
        summary: BenchmarkDataPortfolioAuditSummary {
            corpus_count: 1,
            admitted_corpus_count: 1,
            source_defined_corpus_count: 0,
            missing_corpus_count: 0,
            publishable_data_ready_count: 1,
            bound_catalog_count: 1,
            missing_catalog_binding_count: 0,
            unbound_catalog_count: 0,
            verified_bound_file_count: 1,
            verified_bound_size_bytes: 100,
            unknown_storage_projection_count: 0,
            unknown_compute_projection_count: 1,
            suspected_nonproduction_asset_count: 0,
            publishable_contamination_count: 0,
        },
        corpora: vec![BenchmarkCorpusDataAudit {
            corpus_id: "real-corpus".to_string(),
            status: BenchmarkCorpusStatus::Admitted,
            evidence_scope: BenchmarkEvidenceScope::Comparative,
            class: BenchmarkCorpusClass::ProductionReal,
            declared_sample_count: 10,
            minimum_sample_count: 10,
            data_types: vec!["fastq".to_string()],
            truth_roles: vec!["read_truth".to_string()],
            truth_strategy: "independent observation".to_string(),
            source_specs: Vec::new(),
            catalog_bindings: vec![BenchmarkGovernedDataBinding {
                catalog_id: "real-catalog".to_string(),
                present: true,
                catalog_path: Some("study://data/catalog/real-catalog.json".to_string()),
                catalog_sha256: Some("4".repeat(64)),
                access_classification: Some("public".to_string()),
                redistribution_classification: Some("public".to_string()),
                access_terms: Some("public archive terms".to_string()),
                integrity_verified: true,
                benchmark_eligible: true,
                verified_file_count: 1,
                verified_size_bytes: 100,
                observed_sample_count: Some(10),
                sample_count_authority: Some("catalog.sample_count".to_string()),
                file_roles: vec!["raw-fastq".to_string()],
            }],
            verified_storage_bytes: 100,
            projected_storage_bytes: Some(100),
            projected_cpu_hours: None,
            publishable_data_ready: true,
            blockers: Vec::new(),
        }],
        unbound_assets: Vec::new(),
        publishable_contamination: Vec::new(),
    }
}

#[test]
fn data_portfolio_audit_accepts_exact_ready_closure() -> anyhow::Result<()> {
    validate_benchmark_data_portfolio_audit(&audit())?;
    Ok(())
}

#[test]
fn data_portfolio_audit_rejects_ready_source_defined_corpus() {
    let mut audit = audit();
    audit.corpora[0].status = BenchmarkCorpusStatus::SourceDefined;
    audit.summary.admitted_corpus_count = 0;
    audit.summary.source_defined_corpus_count = 1;

    let error =
        validate_benchmark_data_portfolio_audit(&audit).err().map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("incomplete corpus")));
}

#[test]
fn data_portfolio_audit_rejects_absent_binding_with_evidence() {
    let mut audit = audit();
    audit.corpora[0].publishable_data_ready = false;
    audit.corpora[0].blockers = vec!["catalog missing".to_string()];
    audit.summary.publishable_data_ready_count = 0;
    audit.corpora[0].catalog_bindings[0].present = false;

    let error =
        validate_benchmark_data_portfolio_audit(&audit).err().map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("absent catalog binding")));
}

#[test]
fn data_portfolio_audit_rejects_ready_corpus_without_sample_authority() {
    let mut audit = audit();
    audit.corpora[0].catalog_bindings[0].observed_sample_count = None;
    audit.corpora[0].catalog_bindings[0].sample_count_authority = None;

    let error =
        validate_benchmark_data_portfolio_audit(&audit).err().map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("governed evidence")));
}

#[test]
fn data_portfolio_audit_rejects_fixture_class() {
    let mut audit = audit();
    audit.corpora[0].class = BenchmarkCorpusClass::TestFixture;

    let error =
        validate_benchmark_data_portfolio_audit(&audit).err().map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("not publishable real data")));
}

#[test]
fn checked_data_portfolio_audit_schema_matches_model() -> anyhow::Result<()> {
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(
        repository_root()?.join("benchmarks/contracts/schemas/data-portfolio-audit.schema.json"),
    )?)?;
    let generated = serde_json::to_value(benchmark_data_portfolio_audit_schema())?;
    assert_eq!(checked, generated);
    Ok(())
}

fn repository_root() -> anyhow::Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("repository root")
}
