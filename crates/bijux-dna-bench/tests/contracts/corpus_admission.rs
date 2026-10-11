use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_bench::{
    admit_bam_benchmark_corpus, admit_fastq_benchmark_corpus, BenchmarkFastqLayout,
};
use serde_json::{json, Value};
use tempfile::TempDir;

const CORPUS_ID: &str = "human-short-read-diversity";

struct CorpusFixture {
    _temp: TempDir,
    study_root: PathBuf,
    data_index: PathBuf,
    first_fastq: PathBuf,
}

struct BamCorpusFixture {
    _temp: TempDir,
    study_root: PathBuf,
    data_index: PathBuf,
    bam: PathBuf,
}

#[test]
fn admits_exact_verified_bam_and_index_closure() -> Result<()> {
    let fixture = bam_corpus_fixture()?;
    let admitted =
        admit_bam_benchmark_corpus(&fixture.study_root, &fixture.data_index, "horse-bam")?;

    assert_eq!(admitted.corpus_id, "horse-bam");
    assert_eq!(admitted.sample_id, "ERR2112767");
    assert_eq!(admitted.bam.size_bytes, 9);
    assert_eq!(admitted.bam_index.size_bytes, 5);
    assert_eq!(admitted.input_bytes, 14);
    assert!(Path::new(&admitted.bam.path).is_absolute());
    assert_eq!(admitted.catalog_sha256.len(), 64);
    Ok(())
}

#[test]
fn rejects_bam_bytes_that_drift_after_index_publication() -> Result<()> {
    let fixture = bam_corpus_fixture()?;
    fs::write(&fixture.bam, b"tampered BAM bytes")?;

    let error = admit_bam_benchmark_corpus(&fixture.study_root, &fixture.data_index, "horse-bam")
        .expect_err("tampered BAM bytes must be rejected");
    assert!(error.to_string().contains("size differs"));
    Ok(())
}

#[test]
fn admits_exact_verified_fastq_closure_in_deterministic_order() -> Result<()> {
    let fixture = corpus_fixture()?;
    let admitted =
        admit_fastq_benchmark_corpus(&fixture.study_root, &fixture.data_index, CORPUS_ID, 2, 3)?;

    assert_eq!(admitted.corpus_id, CORPUS_ID);
    assert_eq!(admitted.sample_count, 2);
    assert_eq!(admitted.file_count, 3);
    assert_eq!(admitted.samples[0].sample_id, "ERR000001");
    assert_eq!(admitted.samples[0].input_layout, BenchmarkFastqLayout::PairedEnd);
    assert_eq!(
        admitted.samples[0].files.iter().map(|file| file.mate).collect::<Vec<_>>(),
        vec![Some(1), Some(2)]
    );
    assert_eq!(admitted.samples[1].sample_id, "ERR000002");
    assert_eq!(admitted.samples[1].input_layout, BenchmarkFastqLayout::SingleEnd);
    assert_eq!(admitted.input_bytes, 54);
    assert!(admitted
        .samples
        .iter()
        .flat_map(|sample| &sample.files)
        .all(|file| { Path::new(&file.path).is_absolute() && file.sha256.len() == 64 }));
    Ok(())
}

#[test]
fn rejects_file_bytes_that_drift_after_index_publication() -> Result<()> {
    let fixture = corpus_fixture()?;
    fs::write(&fixture.first_fastq, b"@read-1\nAAAAA\n+\n!!!!!\n")?;

    let error =
        admit_fastq_benchmark_corpus(&fixture.study_root, &fixture.data_index, CORPUS_ID, 2, 3)
            .expect_err("tampered corpus bytes must be rejected");
    assert!(error.to_string().contains("size differs"));
    Ok(())
}

#[test]
fn rejects_catalog_and_index_file_identity_disagreement() -> Result<()> {
    let fixture = corpus_fixture()?;
    let mut index: Value = serde_json::from_slice(&fs::read(&fixture.data_index)?)?;
    index["records"][0]["files"][0]["sha256"] = Value::String("0".repeat(64));
    write_json(&fixture.data_index, &index)?;

    let error =
        admit_fastq_benchmark_corpus(&fixture.study_root, &fixture.data_index, CORPUS_ID, 2, 3)
            .expect_err("cross-authority disagreement must be rejected");
    assert!(error.to_string().contains("catalog and data index disagree"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn rejects_symlink_substitution_even_when_target_bytes_match() -> Result<()> {
    use std::os::unix::fs::symlink;

    let fixture = corpus_fixture()?;
    let replacement = fixture.first_fastq.with_file_name("replacement.fastq");
    fs::rename(&fixture.first_fastq, &replacement)?;
    symlink(&replacement, &fixture.first_fastq)?;

    let error =
        admit_fastq_benchmark_corpus(&fixture.study_root, &fixture.data_index, CORPUS_ID, 2, 3)
            .expect_err("symlinked corpus file must be rejected");
    assert!(error.to_string().contains("regular non-symlink file"));
    Ok(())
}

fn corpus_fixture() -> Result<CorpusFixture> {
    let temp = tempfile::tempdir()?;
    let study_root = temp.path().join("STUDY");
    let catalog_path = study_root.join("data/catalog/human-short-read-diversity.json");
    let data_index = study_root.join("data/evidence/indexes/source/data-index.json");
    let raw_root = study_root.join("data/corpora/human-short-read-diversity/raw");
    fs::create_dir_all(catalog_path.parent().expect("catalog parent"))?;
    fs::create_dir_all(data_index.parent().expect("index parent"))?;

    let files = [
        ("ERR000002/ERR000002.fastq", "ERR000002", "se", None, b"@read-3\nAC\n+\n!!\n".as_slice()),
        (
            "ERR000001/ERR000001_2.fastq",
            "ERR000001",
            "pe",
            Some(2),
            b"@read-2\nACGT\n+\n!!!!\n".as_slice(),
        ),
        (
            "ERR000001/ERR000001_1.fastq",
            "ERR000001",
            "pe",
            Some(1),
            b"@read-1\nACG\n+\n!!!\n".as_slice(),
        ),
    ];
    let mut catalog_files = Vec::new();
    let mut index_files = Vec::new();
    for (relative, accession, layout, mate, bytes) in files {
        let path = raw_root.join(relative);
        fs::create_dir_all(path.parent().expect("FASTQ parent"))?;
        fs::write(&path, bytes)?;
        let sha256 = bijux_dna_infra::hash_bytes_sha256(bytes);
        catalog_files.push(json!({
            "accession": accession,
            "kind": "raw-fastq",
            "layout": layout,
            "mate": mate,
            "materialization_status": "downloaded",
            "path": format!("/archive/{accession}/{}", path.file_name().expect("file name").to_string_lossy()),
            "sha256": sha256,
            "size_bytes": bytes.len(),
        }));
        index_files.push(json!({
            "path": format!("study://data/corpora/human-short-read-diversity/raw/{relative}"),
            "role": "raw-fastq",
            "sha256": sha256,
            "size_bytes": bytes.len(),
        }));
    }
    let catalog = json!({
        "schema_version": "bijux.data_catalog.v1",
        "id": CORPUS_ID,
        "samples": [
            {"accession": "ERR000002", "layout": "se"},
            {"accession": "ERR000001", "layout": "pe"},
        ],
        "files": catalog_files,
    });
    write_json(&catalog_path, &catalog)?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
    let index = json!({
        "schema_version": "bijux.data_index.v1",
        "created_at_utc": "2026-08-05T00:00:00Z",
        "records": [{
            "id": CORPUS_ID,
            "benchmark_eligibility": "eligible",
            "integrity_status": "verified",
            "catalog_path": "study://data/catalog/human-short-read-diversity.json",
            "catalog_sha256": catalog_sha256,
            "files": index_files,
        }],
    });
    write_json(&data_index, &index)?;

    Ok(CorpusFixture {
        first_fastq: raw_root.join("ERR000002/ERR000002.fastq"),
        _temp: temp,
        study_root,
        data_index,
    })
}

fn bam_corpus_fixture() -> Result<BamCorpusFixture> {
    let temp = tempfile::tempdir()?;
    let study_root = temp.path().join("STUDY");
    let catalog_path = study_root.join("data/catalog/horse-bam.json");
    let data_index = study_root.join("data/evidence/indexes/source/data-index.json");
    let corpus_root = study_root.join("data/corpora/horse-bam");
    let bam = corpus_root.join("ERR2112767.sorted.bam");
    let bam_index = corpus_root.join("ERR2112767.sorted.bam.bai");
    fs::create_dir_all(catalog_path.parent().expect("catalog parent"))?;
    fs::create_dir_all(data_index.parent().expect("index parent"))?;
    fs::create_dir_all(&corpus_root)?;
    fs::write(&bam, b"BAM-bytes")?;
    fs::write(&bam_index, b"index")?;
    let bam_sha256 = bijux_dna_infra::hash_file_sha256(&bam)?;
    let index_sha256 = bijux_dna_infra::hash_file_sha256(&bam_index)?;
    let catalog = json!({
        "schema_version": "bijux.data_catalog.v1",
        "id": "horse-bam",
        "run_accession": "ERR2112767",
        "files": [
            {
                "path": bam,
                "kind": "coordinate-sorted-deduplicated-bam",
                "sha256": bam_sha256,
                "size_bytes": 9,
            },
            {
                "path": bam_index,
                "kind": "bam-index",
                "sha256": index_sha256,
                "size_bytes": 5,
            },
        ],
    });
    write_json(&catalog_path, &catalog)?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
    let index = json!({
        "schema_version": "bijux.data_index.v1",
        "records": [{
            "id": "horse-bam",
            "benchmark_eligibility": "eligible",
            "integrity_status": "verified",
            "catalog_path": "study://data/catalog/horse-bam.json",
            "catalog_sha256": catalog_sha256,
            "files": [
                {
                    "path": "study://data/corpora/horse-bam/ERR2112767.sorted.bam",
                    "role": "coordinate-sorted-deduplicated-bam",
                    "sha256": bam_sha256,
                    "size_bytes": 9,
                },
                {
                    "path": "study://data/corpora/horse-bam/ERR2112767.sorted.bam.bai",
                    "role": "bam-index",
                    "sha256": index_sha256,
                    "size_bytes": 5,
                },
            ],
        }],
    });
    write_json(&data_index, &index)?;
    Ok(BamCorpusFixture { _temp: temp, study_root, data_index, bam })
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}
