use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_bench::{
    build_benchmark_data_index, derive_benchmark_data_portfolio_audit,
    publish_benchmark_data_portfolio_audit, render_benchmark_data_portfolio_audit_markdown,
    BenchmarkDataIndexRequest, BenchmarkDataPortfolioAuditPublicationRequest,
    BenchmarkDataPortfolioAuditRequest,
};
use serde_json::json;
use tempfile::TempDir;

#[test]
fn audits_live_portfolio_bindings_without_promoting_missing_corpora() -> Result<()> {
    let fixture = DataAuditFixture::new()?;
    let index = build_benchmark_data_index(&BenchmarkDataIndexRequest {
        catalog_root: fixture.catalog_root.clone(),
        data_root: fixture.data_root.clone(),
        created_at_utc: "2026-08-08T00:00:00Z".to_string(),
    })?;
    let index_path = fixture.data_root.join("evidence/indexes/source/data-index.json");
    fs::create_dir_all(index_path.parent().context("index parent")?)?;
    fs::write(&index_path, serde_json::to_vec_pretty(&index)?)?;

    let request = BenchmarkDataPortfolioAuditRequest {
        repository_root: repository_root()?,
        study_root: fixture.study_root.clone(),
        data_index_path: index_path,
        source_commit: "1".repeat(40),
    };
    let audit = derive_benchmark_data_portfolio_audit(&request)?;

    assert_eq!(audit.summary.corpus_count, 9);
    assert_eq!(audit.summary.admitted_corpus_count, 2);
    assert_eq!(audit.summary.missing_corpus_count, 7);
    assert_eq!(audit.summary.publishable_data_ready_count, 2);
    assert_eq!(audit.summary.bound_catalog_count, 3);
    assert_eq!(audit.summary.unbound_catalog_count, 1);
    assert_eq!(audit.summary.suspected_nonproduction_asset_count, 1);
    assert_eq!(audit.summary.publishable_contamination_count, 0);
    assert!(audit
        .corpora
        .iter()
        .find(|corpus| corpus.corpus_id == "human-short-read-diversity")
        .is_some_and(|corpus| corpus.publishable_data_ready));
    assert!(audit
        .corpora
        .iter()
        .find(|corpus| corpus.corpus_id == "variant-truth-panel")
        .is_some_and(|corpus| {
            !corpus.publishable_data_ready
                && corpus.blockers == ["corpus source selection is missing"]
        }));
    let markdown = render_benchmark_data_portfolio_audit_markdown(&audit);
    assert!(markdown.contains("eligibility proves governed file identity"));
    assert!(markdown.contains("## Corpus inventory and projections"));
    assert!(markdown.contains("## Truth-role matrix"));
    assert!(markdown.contains("## Bound catalog access matrix"));
    assert!(markdown.contains("not declared by governed catalog"));
    assert!(markdown.contains("identity_or_purpose_contains_toy"));
    assert!(markdown.contains("`toy-unbound-asset`"));

    let publication = BenchmarkDataPortfolioAuditPublicationRequest {
        audit: request,
        json_path: fixture.study_root.join("data-portfolio-audit.json"),
        markdown_path: fixture.study_root.join("data-portfolio-audit.md"),
    };
    assert_eq!(publish_benchmark_data_portfolio_audit(&publication)?, audit);
    let error = publish_benchmark_data_portfolio_audit(&publication)
        .err()
        .context("second immutable publication must fail")?;
    assert!(error.to_string().contains("refusing to replace"));
    Ok(())
}

struct DataAuditFixture {
    _temp: TempDir,
    study_root: PathBuf,
    data_root: PathBuf,
    catalog_root: PathBuf,
}

impl DataAuditFixture {
    fn new() -> Result<Self> {
        let temp = tempfile::tempdir()?;
        let study_root = temp.path().join("STUDY");
        let data_root = study_root.join("data");
        let catalog_root = data_root.join("catalog");
        fs::create_dir_all(&catalog_root)?;
        write_catalog(
            &data_root,
            &catalog_root,
            "human-short-read-diversity",
            "real public FASTQ cohort",
            Some(20),
        )?;
        write_catalog(
            &data_root,
            &catalog_root,
            "ena-prjeb22390-err2112767-bam",
            "real ancient horse alignment",
            Some(1),
        )?;
        write_catalog(
            &data_root,
            &catalog_root,
            "equcab3-gcf-002863925.1",
            "real horse reference assembly",
            None,
        )?;
        write_catalog(
            &data_root,
            &catalog_root,
            "toy-unbound-asset",
            "toy fixture excluded from publication",
            Some(1),
        )?;
        Ok(Self { _temp: temp, study_root, data_root, catalog_root })
    }
}

fn write_catalog(
    data_root: &Path,
    catalog_root: &Path,
    id: &str,
    purpose: &str,
    sample_count: Option<u64>,
) -> Result<()> {
    let data_path = data_root.join("corpora").join(id).join("input.dat");
    fs::create_dir_all(data_path.parent().context("data parent")?)?;
    fs::write(&data_path, id.as_bytes())?;
    let digest = bijux_dna_infra::hash_file_sha256(&data_path)?;
    let mut catalog = json!({
        "schema_version": "bijux.data_catalog.v1",
        "id": id,
        "purpose": purpose,
        "source": format!("https://example.invalid/{id}"),
        "access_classification": "public",
        "redistribution_classification": "public",
        "retrieved_at_utc": "2026-08-08T00:00:00Z",
        "known_limitations": ["minimal bytes exercise only the typed join contract"],
        "files": [{
            "path": data_path,
            "sha256": digest,
            "size_bytes": fs::metadata(data_path)?.len(),
            "role": "governed-input"
        }]
    });
    if let Some(sample_count) = sample_count {
        catalog["sample_count"] = json!(sample_count);
    }
    fs::write(catalog_root.join(format!("{id}.json")), serde_json::to_vec_pretty(&catalog)?)?;
    Ok(())
}

fn repository_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("repository root")
}
