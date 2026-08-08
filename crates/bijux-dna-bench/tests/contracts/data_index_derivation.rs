use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_bench::{
    build_benchmark_data_index, publish_benchmark_data_index, BenchmarkDataIndexPublicationRequest,
    BenchmarkDataIndexRequest, BenchmarkEligibility,
};
use serde_json::{json, Value};
use tempfile::TempDir;

struct DataIndexFixture {
    _temp: TempDir,
    data_root: PathBuf,
    catalog_root: PathBuf,
    catalog_path: PathBuf,
    governed_file: PathBuf,
}

#[test]
fn derives_verified_path_stable_index_and_report() -> Result<()> {
    let fixture = data_index_fixture()?;
    let index_request = request(&fixture);
    let first = build_benchmark_data_index(&index_request)?;
    let second = build_benchmark_data_index(&index_request)?;

    assert_eq!(first, second);
    assert_eq!(first.summary.record_count, 1);
    assert_eq!(first.summary.file_count, 1);
    assert_eq!(first.summary.benchmark_eligibility.get(&BenchmarkEligibility::Eligible), Some(&1));
    assert_eq!(first.records[0].files[0].path, "study://data/corpora/horse/sample.bam");

    let index_path = fixture.data_root.join("evidence/indexes/source/data-index.json");
    let report_path = fixture.data_root.join("evidence/indexes/source/data-index.md");
    publish_benchmark_data_index(&BenchmarkDataIndexPublicationRequest {
        index: index_request,
        index_path: index_path.clone(),
        report_path: report_path.clone(),
    })?;
    assert!(index_path.is_file());
    assert!(fs::read_to_string(report_path)?.contains("Catalog records: **1**"));

    let published = fs::read(&index_path)?;
    let duplicate = BenchmarkDataIndexPublicationRequest {
        index: request(&fixture),
        index_path: index_path.clone(),
        report_path: fixture.data_root.join("evidence/indexes/source/replacement.md"),
    };
    assert!(publish_benchmark_data_index(&duplicate)
        .is_err_and(|error| format!("{error:#}").contains("refusing to replace")));
    assert_eq!(fs::read(index_path)?, published);
    Ok(())
}

#[test]
fn exposes_missing_temporal_provenance_without_fabrication() -> Result<()> {
    let fixture = data_index_fixture()?;
    mutate_catalog(&fixture.catalog_path, |record| {
        record.as_object_mut().expect("catalog object").remove("retrieved_at_utc");
    })?;

    let index = build_benchmark_data_index(&request(&fixture))?;
    assert_eq!(index.records[0].benchmark_eligibility, BenchmarkEligibility::Incomplete);
    assert_eq!(index.records[0].missing_provenance, ["temporal_provenance"]);
    Ok(())
}

#[test]
fn rejects_digest_drift_and_ambiguous_bare_names() -> Result<()> {
    let fixture = data_index_fixture()?;
    mutate_catalog(&fixture.catalog_path, |record| {
        record["files"][0]["sha256"] = Value::String("0".repeat(64));
    })?;
    assert!(build_benchmark_data_index(&request(&fixture))
        .is_err_and(|error| format!("{error:#}").contains("digest drifted")));

    let fixture = data_index_fixture()?;
    let duplicate = fixture.data_root.join("references/sample.bam");
    fs::create_dir_all(duplicate.parent().expect("reference parent"))?;
    fs::copy(&fixture.governed_file, duplicate)?;
    let digest = bijux_dna_infra::hash_file_sha256(&fixture.governed_file)?;
    let size = fs::metadata(&fixture.governed_file)?.len();
    mutate_catalog(&fixture.catalog_path, |record| {
        record["files"] = json!([{"name": "sample.bam", "sha256": digest, "size_bytes": size}]);
    })?;
    assert!(build_benchmark_data_index(&request(&fixture))
        .is_err_and(|error| format!("{error:#}").contains("not uniquely resolvable")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn excludes_operational_evidence_from_governed_file_discovery() -> Result<()> {
    use std::os::unix::fs::symlink;

    let fixture = data_index_fixture()?;
    let evidence_root = fixture.data_root.join("evidence/source-checkouts/repository");
    fs::create_dir_all(&evidence_root)?;
    fs::copy(&fixture.governed_file, evidence_root.join("sample.bam"))?;
    symlink(&fixture.governed_file, evidence_root.join("source-link.bam"))?;
    let digest = bijux_dna_infra::hash_file_sha256(&fixture.governed_file)?;
    let size = fs::metadata(&fixture.governed_file)?.len();
    mutate_catalog(&fixture.catalog_path, |record| {
        record["files"] = json!([{"name": "sample.bam", "sha256": digest, "size_bytes": size}]);
    })?;

    let index = build_benchmark_data_index(&request(&fixture))?;
    assert_eq!(index.summary.file_count, 1);
    assert_eq!(index.records[0].files[0].path, "study://data/corpora/horse/sample.bam");
    Ok(())
}

#[test]
fn admits_structured_parent_file_identity_and_provenance() -> Result<()> {
    let fixture = data_index_fixture()?;
    let parent = fixture.data_root.join("corpora/horse/source.cram");
    fs::write(&parent, b"governed-parent")?;
    let digest = bijux_dna_infra::hash_file_sha256(&parent)?;
    let size = fs::metadata(&parent)?.len();
    mutate_catalog(&fixture.catalog_path, |record| {
        record["parents"] = json!([{"path": parent, "sha256": digest, "size_bytes": size}]);
    })?;

    let index = build_benchmark_data_index(&request(&fixture))?;
    assert_eq!(index.summary.file_count, 2);
    assert!(index.records[0]
        .source_identities
        .iter()
        .any(|identity| identity.ends_with("/corpora/horse/source.cram")));
    assert!(index.records[0].files.iter().any(|file| file.path.ends_with("source.cram")));
    Ok(())
}

#[test]
fn derives_legacy_size_but_rejects_incomplete_identity() -> Result<()> {
    let fixture = data_index_fixture()?;
    let parent = fixture.data_root.join("corpora/horse/source.cram");
    fs::write(&parent, b"governed-parent")?;
    let digest = bijux_dna_infra::hash_file_sha256(&parent)?;
    mutate_catalog(&fixture.catalog_path, |record| {
        record["files"].as_array_mut().expect("files").push(json!({
            "path": parent,
            "sha256": digest,
        }));
    })?;
    let index = build_benchmark_data_index(&request(&fixture))?;
    let parent = index.records[0]
        .files
        .iter()
        .find(|file| file.path.ends_with("source.cram"))
        .expect("derived parent file");
    assert_eq!(parent.size_bytes, 15);

    let fixture = data_index_fixture()?;
    mutate_catalog(&fixture.catalog_path, |record| {
        record["files"][0].as_object_mut().expect("file object").remove("sha256");
    })?;
    assert!(build_benchmark_data_index(&request(&fixture))
        .is_err_and(|error| format!("{error:#}").contains("incomplete file identity")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_governed_files() -> Result<()> {
    use std::os::unix::fs::symlink;

    let fixture = data_index_fixture()?;
    let source = fixture.data_root.join("corpora/horse/source.bam");
    fs::rename(&fixture.governed_file, &source)?;
    symlink(&source, &fixture.governed_file)?;
    assert!(build_benchmark_data_index(&request(&fixture))
        .is_err_and(|error| format!("{error:#}").contains("symlink")));
    Ok(())
}

fn data_index_fixture() -> Result<DataIndexFixture> {
    let temp = tempfile::tempdir()?;
    let data_root = temp.path().join("data");
    let catalog_root = data_root.join("catalog");
    let governed_file = data_root.join("corpora/horse/sample.bam");
    fs::create_dir_all(&catalog_root)?;
    fs::create_dir_all(governed_file.parent().expect("corpus parent"))?;
    fs::write(&governed_file, b"governed-bam")?;
    let catalog_path = catalog_root.join("ancient-horse-bam.json");
    let catalog = json!({
        "schema_version": "bijux.data_catalog.v2",
        "id": "ancient-horse-bam",
        "purpose": "real ancient horse alignment",
        "source": "ENA",
        "access_classification": "public",
        "redistribution_classification": "accession-referenced",
        "class": "production_real",
        "access_terms_url": "https://www.ebi.ac.uk/ena/browser/about/policies",
        "attribution": "cite source study and accession",
        "projected_storage_bytes": 12,
        "projected_cpu_hours": 1,
        "projection_basis": "bounded contract-test projection",
        "truth_roles": ["alignment_context"],
        "truth_strategy": "observational_only",
        "retrieved_at_utc": "2026-08-03T00:00:00Z",
        "scientific_name": "Equus caballus",
        "reference_assembly": null,
        "files": [{
            "path": governed_file,
            "sha256": bijux_dna_infra::hash_file_sha256(&governed_file)?,
            "size_bytes": fs::metadata(&governed_file)?.len(),
            "kind": "coordinate-sorted-bam",
        }],
        "known_limitations": ["one sample does not represent a population"],
    });
    write_json(&catalog_path, &catalog)?;
    Ok(DataIndexFixture { _temp: temp, data_root, catalog_root, catalog_path, governed_file })
}

fn request(fixture: &DataIndexFixture) -> BenchmarkDataIndexRequest {
    BenchmarkDataIndexRequest {
        catalog_root: fixture.catalog_root.clone(),
        data_root: fixture.data_root.clone(),
        created_at_utc: "2026-08-06T00:00:00Z".to_string(),
    }
}

fn mutate_catalog(path: &Path, mutation: impl FnOnce(&mut Value)) -> Result<()> {
    let mut value: Value = serde_json::from_slice(&fs::read(path)?)?;
    mutation(&mut value);
    write_json(path, &value)
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}
