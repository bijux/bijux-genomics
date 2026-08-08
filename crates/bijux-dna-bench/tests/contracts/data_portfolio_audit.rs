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
        catalog_registry_path: fixture.catalog_registry_path.clone(),
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
    catalog_registry_path: PathBuf,
}

impl DataAuditFixture {
    fn new() -> Result<Self> {
        let temp = tempfile::tempdir()?;
        let study_root = temp.path().join("STUDY");
        let data_root = study_root.join("data");
        let source_commit = "1".repeat(40);
        let catalog_root = data_root.join("catalog/snapshots").join(&source_commit);
        let catalog_registry_path = data_root.join("catalog/registry.json");
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
        write_catalog_registry(&catalog_registry_path, &catalog_root, &source_commit)?;
        Ok(Self { _temp: temp, study_root, data_root, catalog_registry_path })
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
    let source_catalog = data_root.join("catalog").join(format!("{id}.json"));
    fs::create_dir_all(source_catalog.parent().context("source catalog parent")?)?;
    fs::write(&source_catalog, serde_json::to_vec_pretty(&catalog)?)?;
    let preserved = json!({
        "schema_version": "bijux.data_catalog.v2",
        "id": id,
        "source_catalog_path": format!("study://data/catalog/{id}.json"),
        "source_catalog_sha256": bijux_dna_infra::hash_file_sha256(&source_catalog)?,
        "source_catalog_size_bytes": fs::metadata(&source_catalog)?.len(),
        "source_catalog_schema_version": "bijux.data_catalog.v1",
        "preserved_at_utc": "2026-08-08T00:00:00Z",
        "preserved_catalog": catalog,
    });
    fs::write(catalog_root.join(format!("{id}.json")), serde_json::to_vec_pretty(&preserved)?)?;
    Ok(())
}

fn write_catalog_registry(path: &Path, catalog_root: &Path, source_commit: &str) -> Result<()> {
    let mut catalog_paths = fs::read_dir(catalog_root)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    catalog_paths.sort();
    let records = catalog_paths
        .into_iter()
        .map(|catalog_path| {
            let id = catalog_path
                .file_stem()
                .and_then(|value| value.to_str())
                .context("catalog identity")?;
            let digest = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
            Ok(json!({
                "id": id,
                "current_catalog_sha256": digest,
                "snapshots": [{
                    "catalog_path": format!("study://data/catalog/snapshots/{source_commit}/{id}.json"),
                    "catalog_sha256": digest,
                    "catalog_schema_version": "bijux.data_catalog.v2",
                    "source_commit": source_commit,
                    "created_at_utc": "2026-08-08T00:00:00Z"
                }]
            }))
        })
        .collect::<Result<Vec<_>>>()?;
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "schema_version": "bijux.data_catalog_registry.v1",
            "created_at_utc": "2026-08-08T00:00:00Z",
            "data_root": "study://data",
            "records": records
        }))?,
    )?;
    Ok(())
}

fn repository_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("repository root")
}
