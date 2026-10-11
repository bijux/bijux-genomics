use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_bench::{
    ensure_fastq_profile_reads_expectations, ensure_fastq_read_length_expectations,
    validate_fastq_read_length_expectations,
};
use serde_json::json;
use tempfile::TempDir;

const CORPUS_ID: &str = "exact-fastq-corpus";

struct ExpectationFixture {
    _temp: TempDir,
    repository_root: PathBuf,
    study_root: PathBuf,
    data_index: PathBuf,
}

#[test]
fn publishes_exact_awk_compatible_evidence_and_replays_identically() -> Result<()> {
    let fixture = expectation_fixture()?;
    let first = ensure_fastq_read_length_expectations(
        &fixture.repository_root,
        &fixture.study_root,
        &fixture.data_index,
        CORPUS_ID,
        1,
        1,
    )?;
    let second = ensure_fastq_read_length_expectations(
        &fixture.repository_root,
        &fixture.study_root,
        &fixture.data_index,
        CORPUS_ID,
        1,
        1,
    )?;

    assert_eq!(first, second);
    validate_fastq_read_length_expectations(&first.expectations)?;
    assert_eq!(
        first.root.parent().and_then(Path::file_name).and_then(|value| value.to_str()),
        Some(first.expectations.derivation_sha256.as_str())
    );
    assert_eq!(
        fs::read_to_string(first.root.join("summaries/ERR000001.summary.tsv"))?,
        "sample_id\tinput_layout\tfile_count\tread_count\tminimum_read_length\tmean_read_length\tmedian_read_length\tmaximum_read_length\tdistinct_read_lengths\nERR000001\tsingle_end\t1\t2\t2\t3\t3\t4\t2\n"
    );
    assert_eq!(
        fs::read_to_string(first.root.join("histograms/ERR000001.histogram.tsv"))?,
        "sample_id\tread_length\tcount\nERR000001\t2\t1\nERR000001\t4\t1\n"
    );
    assert_eq!(first.expectations.samples[0].read_count, 2);
    assert!(first.manifest_path.is_file());
    let mut invalid = first.expectations.clone();
    invalid.samples[0].summary_path = "../escaped.tsv".to_string();
    assert!(validate_fastq_read_length_expectations(&invalid)
        .is_err_and(|error| error.to_string().contains("artifact path")));
    Ok(())
}

#[test]
fn publishes_exact_read_and_base_count_admission_profile() -> Result<()> {
    let fixture = expectation_fixture()?;
    let read_lengths = ensure_fastq_read_length_expectations(
        &fixture.repository_root,
        &fixture.study_root,
        &fixture.data_index,
        CORPUS_ID,
        1,
        1,
    )?;
    let first = ensure_fastq_profile_reads_expectations(&fixture.study_root, &read_lengths)?;
    let second = ensure_fastq_profile_reads_expectations(&fixture.study_root, &read_lengths)?;

    assert_eq!(first, second);
    assert_eq!(first.samples[0].read_count, 2);
    assert_eq!(first.samples[0].base_count, 6);
    assert_eq!(
        fs::read_to_string(&first.profile_path)?,
        "sample_id\tinput_layout\tfile_count\tread_count\tbase_count\nERR000001\tsingle_end\t1\t2\t6\n"
    );
    assert_eq!(
        first.root.file_name().and_then(|value| value.to_str()),
        Some("fastq-profile-reads")
    );
    Ok(())
}

#[test]
fn rejects_conflicting_existing_read_profile_truth() -> Result<()> {
    let fixture = expectation_fixture()?;
    let read_lengths = ensure_fastq_read_length_expectations(
        &fixture.repository_root,
        &fixture.study_root,
        &fixture.data_index,
        CORPUS_ID,
        1,
        1,
    )?;
    let publication = ensure_fastq_profile_reads_expectations(&fixture.study_root, &read_lengths)?;
    fs::write(&publication.profile_path, b"conflict\n")?;

    let error = ensure_fastq_profile_reads_expectations(&fixture.study_root, &read_lengths)
        .expect_err("conflicting profile truth must be rejected");
    assert!(error.to_string().contains("conflicts with immutable evidence"));
    Ok(())
}

#[test]
fn rejects_conflicting_existing_expectation_bytes() -> Result<()> {
    let fixture = expectation_fixture()?;
    let publication = ensure_fastq_read_length_expectations(
        &fixture.repository_root,
        &fixture.study_root,
        &fixture.data_index,
        CORPUS_ID,
        1,
        1,
    )?;
    fs::write(publication.root.join("summaries/ERR000001.summary.tsv"), b"conflict\n")?;

    let error = ensure_fastq_read_length_expectations(
        &fixture.repository_root,
        &fixture.study_root,
        &fixture.data_index,
        CORPUS_ID,
        1,
        1,
    )
    .expect_err("conflicting expectation evidence must be rejected");
    assert!(error.to_string().contains("conflicts with immutable evidence"));
    Ok(())
}

#[test]
fn rejects_extra_artifacts_outside_the_manifest_closure() -> Result<()> {
    let fixture = expectation_fixture()?;
    let publication = ensure_fastq_read_length_expectations(
        &fixture.repository_root,
        &fixture.study_root,
        &fixture.data_index,
        CORPUS_ID,
        1,
        1,
    )?;
    fs::write(publication.root.join("histograms/untracked.tsv"), b"untracked\n")?;

    let error = ensure_fastq_read_length_expectations(
        &fixture.repository_root,
        &fixture.study_root,
        &fixture.data_index,
        CORPUS_ID,
        1,
        1,
    )
    .expect_err("extra expectation evidence must be rejected");
    assert!(error.to_string().contains("directory closure differs"));
    Ok(())
}

fn expectation_fixture() -> Result<ExpectationFixture> {
    let temp = tempfile::tempdir()?;
    let study_root = temp.path().join("STUDY");
    let catalog_path = study_root.join("data/catalog/exact-fastq-corpus.json");
    let data_index = study_root.join("data/evidence/indexes/source/data-index.json");
    let fastq_path = study_root.join("data/corpora/exact-fastq-corpus/raw/ERR000001/reads.fastq");
    fs::create_dir_all(catalog_path.parent().expect("catalog parent"))?;
    fs::create_dir_all(data_index.parent().expect("index parent"))?;
    fs::create_dir_all(fastq_path.parent().expect("FASTQ parent"))?;
    let fastq = b"@r1\nAC\n+\n!!\n@r2\nACGT\n+\n!!!!\n";
    fs::write(&fastq_path, fastq)?;
    let fastq_sha256 = bijux_dna_infra::hash_bytes_sha256(fastq);
    let catalog = json!({
        "schema_version": "bijux.data_catalog.v1",
        "id": CORPUS_ID,
        "samples": [{"accession": "ERR000001", "layout": "se"}],
        "files": [{
            "accession": "ERR000001",
            "kind": "raw-fastq",
            "layout": "se",
            "mate": null,
            "materialization_status": "downloaded",
            "path": "/archive/ERR000001/reads.fastq",
            "sha256": fastq_sha256,
            "size_bytes": fastq.len(),
        }],
    });
    write_json(&catalog_path, &catalog)?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
    let index = json!({
        "schema_version": "bijux.data_index.v1",
        "records": [{
            "id": CORPUS_ID,
            "benchmark_eligibility": "eligible",
            "integrity_status": "verified",
            "catalog_path": "study://data/catalog/exact-fastq-corpus.json",
            "catalog_sha256": catalog_sha256,
            "files": [{
                "path": "study://data/corpora/exact-fastq-corpus/raw/ERR000001/reads.fastq",
                "role": "raw-fastq",
                "sha256": fastq_sha256,
                "size_bytes": fastq.len(),
            }],
        }],
    });
    write_json(&data_index, &index)?;
    Ok(ExpectationFixture {
        _temp: temp,
        repository_root: super::support::repo_root()?,
        study_root,
        data_index,
    })
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}
