//! Owner: bijux-dna-bench
//! Generic corpus-snapshot admission for registry-driven benchmark cases.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_data_index, validate_governed_data_catalog_document,
    validate_governed_data_catalog_registry, BenchmarkCorpusSnapshotV1, BenchmarkDataIndex,
    BenchmarkDataIndexRecord, BenchmarkEligibility, DataIntegrityStatus,
    GovernedDataCatalogDocument, GovernedDataCatalogRegistry,
};

const MAX_AUTHORITY_BYTES: usize = 64 * 1024 * 1024;

/// Derive exact case-planner corpus snapshots from a governed v2 data index.
///
/// # Errors
/// Returns an error when source authorities, selected records, catalogs, files, or sample
/// identities are missing, ambiguous, unsafe, or differ from their admitted digests.
pub fn derive_benchmark_corpus_snapshots(
    study_root: &Path,
    data_index_path: &Path,
    corpus_ids: &[String],
) -> Result<Vec<BenchmarkCorpusSnapshotV1>> {
    let study_root = std::fs::canonicalize(study_root)
        .with_context(|| format!("resolve STUDY root {}", study_root.display()))?;
    let index_path = governed_regular_file(&study_root, data_index_path, "data index")?;
    let index: BenchmarkDataIndex = read_json(&index_path, "data index")?;
    validate_benchmark_data_index(&index).context("validate benchmark data index")?;
    let registry_uri = index
        .catalog_registry_path
        .as_deref()
        .context("benchmark case planning requires a v2 data index")?;
    let registry_sha256 = index
        .catalog_registry_sha256
        .as_deref()
        .context("benchmark case planning requires a v2 catalog registry digest")?;
    let registry_path = governed_regular_file(
        &study_root,
        &resolve_study_uri(&study_root, registry_uri, "catalog registry")?,
        "catalog registry",
    )?;
    let registry_bytes = read_authority(&registry_path, "catalog registry")?;
    if bijux_dna_infra::hash_bytes_sha256(&registry_bytes) != registry_sha256 {
        bail!("catalog registry digest differs from the data index");
    }
    let registry: GovernedDataCatalogRegistry =
        serde_json::from_slice(&registry_bytes).context("parse governed data catalog registry")?;
    validate_governed_data_catalog_registry(&registry)
        .context("validate governed data catalog registry")?;

    let requested = corpus_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if requested.is_empty() || requested.len() != corpus_ids.len() {
        bail!("benchmark case corpus identities are empty or duplicated");
    }
    let records =
        index.records.iter().map(|record| (record.id.as_str(), record)).collect::<BTreeMap<_, _>>();
    let registry_records = registry
        .records
        .iter()
        .map(|record| (record.id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    requested
        .into_iter()
        .map(|corpus_id| {
            let record = records
                .get(corpus_id)
                .with_context(|| format!("data index lacks benchmark corpus {corpus_id}"))?;
            let registry_record = registry_records
                .get(corpus_id)
                .with_context(|| format!("catalog registry lacks benchmark corpus {corpus_id}"))?;
            if registry_record.current_catalog_sha256 != record.catalog_sha256 {
                bail!("data index and catalog registry disagree for {corpus_id}");
            }
            admit_snapshot(&study_root, record, registry_record)
        })
        .collect()
}

fn admit_snapshot(
    study_root: &Path,
    record: &BenchmarkDataIndexRecord,
    registry_record: &bijux_dna_bench_model::GovernedDataCatalogRegistryRecord,
) -> Result<BenchmarkCorpusSnapshotV1> {
    if record.benchmark_eligibility != BenchmarkEligibility::Eligible
        || record.integrity_status != DataIntegrityStatus::Verified
    {
        bail!("benchmark corpus is not eligible with verified integrity: {}", record.id);
    }
    let snapshot = registry_record
        .snapshots
        .iter()
        .find(|snapshot| snapshot.catalog_sha256 == registry_record.current_catalog_sha256)
        .context("current catalog snapshot is absent")?;
    if snapshot.catalog_path != record.catalog_path {
        bail!("data index and catalog registry paths disagree for {}", record.id);
    }
    let catalog_path = governed_regular_file(
        study_root,
        &resolve_study_uri(study_root, &record.catalog_path, "catalog")?,
        "catalog",
    )?;
    let catalog_bytes = read_authority(&catalog_path, "catalog")?;
    if bijux_dna_infra::hash_bytes_sha256(&catalog_bytes) != record.catalog_sha256 {
        bail!("benchmark corpus catalog digest drifted: {}", record.id);
    }
    let catalog: GovernedDataCatalogDocument =
        serde_json::from_slice(&catalog_bytes).context("parse governed catalog document")?;
    validate_governed_data_catalog_document(&catalog).context("validate governed catalog")?;
    if catalog_id(&catalog) != record.id {
        bail!("benchmark corpus catalog identity drifted: {}", record.id);
    }
    for file in &record.files {
        let path = governed_regular_file(
            study_root,
            &resolve_study_uri(study_root, &file.path, "corpus file")?,
            "corpus file",
        )?;
        if std::fs::metadata(&path)?.len() != file.size_bytes
            || bijux_dna_infra::hash_file_sha256(&path)? != file.sha256
        {
            bail!("benchmark corpus file identity drifted: {}", file.path);
        }
    }
    let sample_ids = catalog_sample_ids(&catalog)?;
    Ok(BenchmarkCorpusSnapshotV1 {
        corpus_id: record.id.clone(),
        snapshot_sha256: record.catalog_sha256.clone(),
        sample_ids,
    })
}

fn catalog_id(catalog: &GovernedDataCatalogDocument) -> &str {
    match catalog {
        GovernedDataCatalogDocument::Materialized(catalog) => &catalog.id,
        GovernedDataCatalogDocument::Preserved(catalog) => &catalog.id,
    }
}

fn catalog_sample_ids(catalog: &GovernedDataCatalogDocument) -> Result<Vec<String>> {
    let mut samples = BTreeSet::new();
    match catalog {
        GovernedDataCatalogDocument::Materialized(catalog) => {
            samples.extend(catalog.samples.iter().map(|sample| sample.accession.clone()));
        }
        GovernedDataCatalogDocument::Preserved(catalog) => {
            let value = catalog
                .preserved_catalog
                .as_object()
                .context("preserved catalog payload is not an object")?;
            if let Some(run) = value.get("run_accession").and_then(serde_json::Value::as_str) {
                samples.insert(run.to_string());
            } else if let Some(sample) =
                value.get("sample_accession").and_then(serde_json::Value::as_str)
            {
                samples.insert(sample.to_string());
            } else if let Some(entries) = value.get("samples").and_then(serde_json::Value::as_array)
            {
                for entry in entries {
                    let accession = entry
                        .get("accession")
                        .and_then(serde_json::Value::as_str)
                        .context("preserved catalog sample lacks accession")?;
                    samples.insert(accession.to_string());
                }
            }
        }
    }
    if samples.is_empty() {
        bail!("governed benchmark corpus has no sample identities");
    }
    Ok(samples.into_iter().collect())
}

fn governed_regular_file(study_root: &Path, path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular non-symlink file: {}", path.display());
    }
    let canonical = std::fs::canonicalize(path)
        .with_context(|| format!("canonicalize {label} {}", path.display()))?;
    if !canonical.starts_with(study_root) {
        bail!("{label} escapes STUDY root: {}", path.display());
    }
    Ok(canonical)
}

fn resolve_study_uri(study_root: &Path, uri: &str, label: &str) -> Result<PathBuf> {
    let relative = uri
        .strip_prefix("study://data/")
        .with_context(|| format!("{label} is not a canonical STUDY data URI"))?;
    let relative = Path::new(relative);
    if relative.components().any(|component| !matches!(component, Component::Normal(_))) {
        bail!("{label} contains an unsafe path");
    }
    Ok(study_root.join("data").join(relative))
}

fn read_authority(path: &Path, label: &str) -> Result<Vec<u8>> {
    bijux_dna_infra::read_to_end_bounded(path, MAX_AUTHORITY_BYTES)
        .with_context(|| format!("read {label} {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T> {
    serde_json::from_slice(&read_authority(path, label)?)
        .with_context(|| format!("parse {label} {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};
    use serde::Serialize;
    use serde_json::json;
    use tempfile::TempDir;

    use super::derive_benchmark_corpus_snapshots;

    struct Fixture {
        _temp: TempDir,
        study_root: PathBuf,
        data_index: PathBuf,
        corpus_file: PathBuf,
    }

    #[test]
    fn derives_source_bound_snapshot_and_rejects_file_drift() -> Result<()> {
        let fixture = fixture()?;
        let snapshots = derive_benchmark_corpus_snapshots(
            &fixture.study_root,
            &fixture.data_index,
            &["horse-bam".to_string()],
        )?;
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].corpus_id, "horse-bam");
        assert_eq!(snapshots[0].sample_ids, ["ERR1"]);

        fs::write(&fixture.corpus_file, b"drifted")?;
        assert!(derive_benchmark_corpus_snapshots(
            &fixture.study_root,
            &fixture.data_index,
            &["horse-bam".to_string()],
        )
        .is_err_and(|error| format!("{error:#}").contains("file identity drifted")));
        Ok(())
    }

    fn fixture() -> Result<Fixture> {
        let temp = tempfile::tempdir()?;
        let study_root = temp.path().join("STUDY");
        let data_root = study_root.join("data");
        let source_commit = "a".repeat(40);
        let corpus_file = data_root.join("corpora/horse/ERR1.bam");
        fs::create_dir_all(corpus_file.parent().context("corpus parent")?)?;
        fs::write(&corpus_file, b"BAM")?;
        let corpus_sha256 = bijux_dna_infra::hash_file_sha256(&corpus_file)?;
        let catalog_path =
            data_root.join("catalog/snapshots").join(&source_commit).join("horse-bam.json");
        write_json(
            &catalog_path,
            &json!({
                "schema_version": "bijux.data_catalog.v2",
                "id": "horse-bam",
                "source_catalog_path": "study://data/catalog/horse-bam.json",
                "source_catalog_sha256": "b".repeat(64),
                "source_catalog_size_bytes": 1,
                "source_catalog_schema_version": "bijux.data_catalog.v1",
                "preserved_at_utc": "2026-08-09T00:00:00Z",
                "preserved_catalog": {
                    "schema_version": "bijux.data_catalog.v1",
                    "id": "horse-bam",
                    "purpose": "real corpus fixture",
                    "access_classification": "public",
                    "redistribution_classification": "allowed",
                    "known_limitations": ["single sample"],
                    "run_accession": "ERR1"
                }
            }),
        )?;
        let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
        let catalog_uri = format!("study://data/catalog/snapshots/{source_commit}/horse-bam.json");
        let registry_path =
            data_root.join("catalog/registries").join(format!("{source_commit}.json"));
        write_json(
            &registry_path,
            &json!({
                "schema_version": "bijux.data_catalog_registry.v1",
                "created_at_utc": "2026-08-09T00:00:00Z",
                "data_root": "study://data",
                "records": [{
                    "id": "horse-bam",
                    "current_catalog_sha256": catalog_sha256,
                    "snapshots": [{
                        "catalog_path": catalog_uri,
                        "catalog_sha256": catalog_sha256,
                        "catalog_schema_version": "bijux.data_catalog.v2",
                        "source_commit": source_commit,
                        "created_at_utc": "2026-08-09T00:00:00Z"
                    }]
                }]
            }),
        )?;
        let registry_sha256 = bijux_dna_infra::hash_file_sha256(&registry_path)?;
        let data_index = data_root.join("evidence/indexes/source/data-index.json");
        write_json(
            &data_index,
            &json!({
                "schema_version": "bijux.data_index.v2",
                "created_at_utc": "2026-08-09T00:00:00Z",
                "data_root": "study://data",
                "catalog_registry_path": format!("study://data/catalog/registries/{source_commit}.json"),
                "catalog_registry_sha256": registry_sha256,
                "summary": {
                    "record_count": 1,
                    "file_count": 1,
                    "size_bytes": 3,
                    "integrity_verified_count": 1,
                    "benchmark_eligibility": {"eligible": 1}
                },
                "records": [{
                    "id": "horse-bam",
                    "catalog_path": catalog_uri,
                    "catalog_sha256": catalog_sha256,
                    "purpose": "real corpus fixture",
                    "source_identities": ["source:fixture"],
                    "access_classification": "public",
                    "redistribution_classification": "allowed",
                    "scientific_name": "Equus caballus",
                    "reference_assembly": "EquCab3.0",
                    "temporal_provenance": {
                        "field": "retrieved_at_utc",
                        "timestamp": "2026-08-09T00:00:00Z"
                    },
                    "files": [{
                        "path": "study://data/corpora/horse/ERR1.bam",
                        "sha256": corpus_sha256,
                        "size_bytes": 3,
                        "role": "coordinate-sorted-bam",
                        "legacy_fields": ["path", "sha256", "size_bytes"]
                    }],
                    "known_limitations": ["single sample"],
                    "integrity_status": "verified",
                    "benchmark_eligibility": "eligible",
                    "missing_provenance": []
                }]
            }),
        )?;
        Ok(Fixture { _temp: temp, study_root, data_index, corpus_file })
    }

    fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
        fs::create_dir_all(path.parent().context("JSON parent")?)?;
        fs::write(path, serde_json::to_vec_pretty(value)?)?;
        Ok(())
    }
}
