use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_bench::{
    admit_bam_benchmark_cohort, admit_bam_benchmark_corpus, admit_bam_genotyping_corpus,
    admit_benchmark_reference, admit_fastq_benchmark_corpus, BenchmarkFastqLayout,
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

struct BamGenotypingCorpusFixture {
    _temp: TempDir,
    study_root: PathBuf,
    data_index: PathBuf,
}

struct BamCohortFixture {
    _temp: TempDir,
    study_root: PathBuf,
    data_index: PathBuf,
    first_bam: PathBuf,
}

struct BenchmarkReferenceFixture {
    _temp: TempDir,
    study_root: PathBuf,
    data_index: PathBuf,
    dictionary: PathBuf,
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
fn admits_bam_from_source_addressed_catalog_snapshot() -> Result<()> {
    let fixture = bam_corpus_fixture()?;
    let catalog_path = fixture.study_root.join("data/catalog/horse-bam.json");
    let source_bytes = fs::read(&catalog_path)?;
    let preserved_catalog: Value = serde_json::from_slice(&source_bytes)?;
    write_json(
        &catalog_path,
        &json!({
            "schema_version": "bijux.data_catalog.v2",
            "id": "horse-bam",
            "source_catalog_path": "study://data/catalog/horse-bam.json",
            "source_catalog_sha256": bijux_dna_infra::hash_bytes_sha256(&source_bytes),
            "source_catalog_size_bytes": source_bytes.len(),
            "source_catalog_schema_version": "bijux.data_catalog.v1",
            "preserved_at_utc": "2026-08-09T00:00:00Z",
            "preserved_catalog": preserved_catalog,
        }),
    )?;
    let mut index: Value = serde_json::from_slice(&fs::read(&fixture.data_index)?)?;
    index["records"][0]["catalog_sha256"] =
        Value::String(bijux_dna_infra::hash_file_sha256(&catalog_path)?);
    write_json(&fixture.data_index, &index)?;

    let admitted =
        admit_bam_benchmark_corpus(&fixture.study_root, &fixture.data_index, "horse-bam")?;
    assert_eq!(admitted.sample_id, "ERR2112767");
    assert_eq!(admitted.input_bytes, 14);
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
fn admits_verified_bam_cohort_in_sample_identity_order() -> Result<()> {
    let fixture = bam_cohort_fixture()?;
    let admitted = admit_bam_benchmark_cohort(
        &fixture.study_root,
        &fixture.data_index,
        "ancient-bam-cohort",
        2,
    )?;

    assert_eq!(admitted.sample_count, 2);
    assert_eq!(admitted.file_count, 4);
    assert_eq!(admitted.samples[0].sample_id, "ERR000001");
    assert_eq!(admitted.samples[1].sample_id, "ERR000002");
    assert_eq!(admitted.samples[0].input_bytes, 14);
    assert_eq!(admitted.samples[1].input_bytes, 14);
    assert_eq!(admitted.input_bytes, 28);
    assert!(admitted.samples.iter().all(|sample| {
        Path::new(&sample.bam.path).is_absolute() && Path::new(&sample.bam_index.path).is_absolute()
    }));
    Ok(())
}

#[test]
fn rejects_bam_cohort_bytes_that_drift_after_index_publication() -> Result<()> {
    let fixture = bam_cohort_fixture()?;
    fs::write(&fixture.first_bam, b"tampered cohort BAM bytes")?;

    let error = admit_bam_benchmark_cohort(
        &fixture.study_root,
        &fixture.data_index,
        "ancient-bam-cohort",
        2,
    )
    .expect_err("tampered cohort BAM bytes must be rejected");
    assert!(error.to_string().contains("size differs"));
    Ok(())
}

#[test]
fn admits_complete_reference_bundle() -> Result<()> {
    let fixture = benchmark_reference_fixture()?;
    let admitted =
        admit_benchmark_reference(&fixture.study_root, &fixture.data_index, "grch38-reference")?;

    assert_eq!(admitted.reference_id, "grch38-reference");
    assert!(admitted.fasta.path.ends_with("GRCh38.fna"));
    assert!(admitted.fasta_index.path.ends_with("GRCh38.fna.fai"));
    assert!(admitted.sequence_dictionary.path.ends_with("GRCh38.dict"));
    assert_eq!(admitted.input_bytes, 35);
    Ok(())
}

#[test]
fn rejects_reference_bytes_that_drift_after_index_publication() -> Result<()> {
    let fixture = benchmark_reference_fixture()?;
    fs::write(&fixture.dictionary, b"tampered sequence dictionary")?;

    let error =
        admit_benchmark_reference(&fixture.study_root, &fixture.data_index, "grch38-reference")
            .expect_err("tampered reference dictionary must be rejected");
    assert!(error.to_string().contains("size differs"));
    Ok(())
}

#[test]
fn admits_complete_bam_genotyping_authority_closure() -> Result<()> {
    let fixture = bam_genotyping_corpus_fixture()?;
    let admitted = admit_bam_genotyping_corpus(&fixture.study_root, &fixture.data_index)?;

    assert_eq!(admitted.bam.sample_id, "ERR2112767");
    assert_eq!(admitted.bam_input_bytes, 14);
    assert_eq!(admitted.reference_input_bytes, 16);
    assert_eq!(admitted.database_input_bytes, 37);
    assert_eq!(admitted.input_bytes, 67);
    Ok(())
}

#[test]
fn admits_preserved_catalog_index_roles_for_genotyping_reference() -> Result<()> {
    let fixture = bam_genotyping_corpus_fixture()?;
    let mut index: Value = serde_json::from_slice(&fs::read(&fixture.data_index)?)?;
    let reference = index["records"]
        .as_array_mut()
        .expect("records array")
        .iter_mut()
        .find(|record| record["id"] == "equcab3-gcf-002863925.1")
        .expect("reference record");
    for file in reference["files"].as_array_mut().expect("reference files") {
        let role = file["role"].as_str().expect("reference role");
        file["role"] = Value::String(role.replacen("catalog.", "catalog.preserved_catalog.", 1));
    }
    write_json(&fixture.data_index, &index)?;

    let admitted = admit_bam_genotyping_corpus(&fixture.study_root, &fixture.data_index)?;
    assert!(admitted.reference.path.ends_with("EquCab3.0.fna"));
    assert!(admitted.reference_index.path.ends_with("EquCab3.0.fna.fai"));
    Ok(())
}

#[test]
fn rejects_genotyping_index_identity_that_disagrees_with_catalog() -> Result<()> {
    let fixture = bam_genotyping_corpus_fixture()?;
    let mut index: Value = serde_json::from_slice(&fs::read(&fixture.data_index)?)?;
    let records = index["records"].as_array_mut().expect("records array");
    let known_sites = records
        .iter_mut()
        .find(|record| record["id"] == "ensembl-113-equcab3-refseq-known-sites-10mb")
        .expect("known-sites record");
    known_sites["files"][1]["sha256"] = Value::String("f".repeat(64));
    write_json(&fixture.data_index, &index)?;

    let error = admit_bam_genotyping_corpus(&fixture.study_root, &fixture.data_index)
        .expect_err("cross-authority genotyping disagreement must be rejected");
    assert!(error.to_string().contains("catalog and data index disagree"));
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
fn admits_fastq_closure_reverified_from_existing_governed_bytes() -> Result<()> {
    let fixture = corpus_fixture()?;
    let catalog_path = fixture.study_root.join("data/catalog/human-short-read-diversity.json");
    let mut catalog: Value = serde_json::from_slice(&fs::read(&catalog_path)?)?;
    for file in catalog["files"].as_array_mut().expect("catalog files") {
        file["materialization_status"] = Value::String("verified_existing".to_string());
    }
    write_json(&catalog_path, &catalog)?;
    let mut index: Value = serde_json::from_slice(&fs::read(&fixture.data_index)?)?;
    index["records"][0]["catalog_sha256"] =
        Value::String(bijux_dna_infra::hash_file_sha256(&catalog_path)?);
    write_json(&fixture.data_index, &index)?;

    let admitted =
        admit_fastq_benchmark_corpus(&fixture.study_root, &fixture.data_index, CORPUS_ID, 2, 3)?;

    assert_eq!(admitted.sample_count, 2);
    assert_eq!(admitted.file_count, 3);
    assert_eq!(admitted.input_bytes, 54);
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
    index["records"][0]["files"][0]["sha256"] = Value::String("f".repeat(64));
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
            "legacy_fields": ["path", "sha256", "size_bytes"],
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
    let index = governed_data_index(vec![json!({
        "id": CORPUS_ID,
        "benchmark_eligibility": "eligible",
        "integrity_status": "verified",
        "catalog_path": "study://data/catalog/human-short-read-diversity.json",
        "catalog_sha256": catalog_sha256,
        "purpose": "exercise FASTQ corpus admission",
        "source_identities": ["source:test-fixture"],
        "access_classification": "test-only",
        "redistribution_classification": "test-only",
        "scientific_name": "Homo sapiens",
        "reference_assembly": null,
        "temporal_provenance": {
            "field": "created_at_utc",
            "timestamp": "2026-08-05T00:00:00Z"
        },
        "files": index_files,
        "known_limitations": ["synthetic test data"],
        "missing_provenance": [],
    })]);
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
        "purpose": "exercise BAM corpus admission",
        "access_classification": "test-only",
        "redistribution_classification": "test-only",
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
        "known_limitations": ["synthetic test data"],
    });
    write_json(&catalog_path, &catalog)?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
    let index = governed_data_index(vec![json!({
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
                "legacy_fields": ["path", "sha256", "size_bytes"],
            },
            {
                "path": "study://data/corpora/horse-bam/ERR2112767.sorted.bam.bai",
                "role": "bam-index",
                "sha256": index_sha256,
                "size_bytes": 5,
                "legacy_fields": ["path", "sha256", "size_bytes"],
            },
        ],
        "purpose": "exercise BAM corpus admission",
        "source_identities": ["source:test-fixture"],
        "access_classification": "test-only",
        "redistribution_classification": "test-only",
        "scientific_name": "Equus caballus",
        "reference_assembly": "EquCab3.0",
        "temporal_provenance": {
            "field": "created_at_utc",
            "timestamp": "2026-08-05T00:00:00Z"
        },
        "known_limitations": ["synthetic test data"],
        "missing_provenance": [],
    })]);
    write_json(&data_index, &index)?;
    Ok(BamCorpusFixture { _temp: temp, study_root, data_index, bam })
}

fn bam_cohort_fixture() -> Result<BamCohortFixture> {
    let temp = tempfile::tempdir()?;
    let study_root = temp.path().join("STUDY");
    let catalog_path = study_root.join("data/catalog/ancient-bam-cohort.json");
    let data_index = study_root.join("data/evidence/indexes/source/data-index.json");
    let corpus_root = study_root.join("data/corpora/ancient-bam-cohort");
    fs::create_dir_all(catalog_path.parent().expect("catalog parent"))?;
    fs::create_dir_all(data_index.parent().expect("index parent"))?;

    let mut catalog_files = Vec::new();
    let mut index_files = Vec::new();
    let mut first_bam = None;
    for sample_id in ["ERR000002", "ERR000001"] {
        let sample_root = corpus_root.join(sample_id);
        fs::create_dir_all(&sample_root)?;
        for (suffix, role, bytes) in [
            (".sorted.bam", "coordinate-sorted-deduplicated-bam", b"BAM-bytes".as_slice()),
            (".sorted.bam.bai", "bam-index", b"index".as_slice()),
        ] {
            let path = sample_root.join(format!("{sample_id}{suffix}"));
            fs::write(&path, bytes)?;
            if sample_id == "ERR000001" && role == "coordinate-sorted-deduplicated-bam" {
                first_bam = Some(path.clone());
            }
            let sha256 = bijux_dna_infra::hash_bytes_sha256(bytes);
            catalog_files.push(json!({
                "accession": sample_id,
                "path": format!("/archive/{sample_id}/{}", path.file_name().expect("file name").to_string_lossy()),
                "kind": role,
                "sha256": sha256,
                "size_bytes": bytes.len(),
            }));
            index_files.push(json!({
                "path": format!("study://data/corpora/ancient-bam-cohort/{sample_id}/{}", path.file_name().expect("file name").to_string_lossy()),
                "role": role,
                "sha256": sha256,
                "size_bytes": bytes.len(),
                "legacy_fields": ["path", "sha256", "size_bytes"],
            }));
        }
    }
    write_json(
        &catalog_path,
        &json!({
            "schema_version": "bijux.data_catalog.v1",
            "id": "ancient-bam-cohort",
            "samples": [
                {"accession": "ERR000002"},
                {"accession": "ERR000001"},
            ],
            "files": catalog_files,
        }),
    )?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
    let index = governed_data_index(vec![json!({
        "id": "ancient-bam-cohort",
        "benchmark_eligibility": "eligible",
        "integrity_status": "verified",
        "catalog_path": "study://data/catalog/ancient-bam-cohort.json",
        "catalog_sha256": catalog_sha256,
        "files": index_files,
        "purpose": "exercise BAM cohort admission",
        "source_identities": ["source:test-fixture"],
        "access_classification": "test-only",
        "redistribution_classification": "test-only",
        "scientific_name": "Homo sapiens",
        "reference_assembly": "GRCh38",
        "temporal_provenance": {
            "field": "created_at_utc",
            "timestamp": "2026-08-11T00:00:00Z"
        },
        "known_limitations": ["synthetic test data"],
        "missing_provenance": [],
    })]);
    write_json(&data_index, &index)?;
    Ok(BamCohortFixture {
        first_bam: first_bam.expect("first BAM"),
        _temp: temp,
        study_root,
        data_index,
    })
}

fn benchmark_reference_fixture() -> Result<BenchmarkReferenceFixture> {
    let temp = tempfile::tempdir()?;
    let study_root = temp.path().join("STUDY");
    let data_index = study_root.join("data/evidence/indexes/source/data-index.json");
    let reference_root = study_root.join("data/references/grch38-reference");
    let catalog_root = study_root.join("data/catalog");
    fs::create_dir_all(data_index.parent().expect("index parent"))?;
    fs::create_dir_all(&reference_root)?;
    fs::create_dir_all(&catalog_root)?;

    let dictionary = reference_root.join("GRCh38.dict");
    let files = write_authority_files(
        &study_root,
        &[
            (reference_root.join("GRCh38.fna"), "reference-fasta", b">chr1\nACGT\n".as_slice()),
            (
                reference_root.join("GRCh38.fna.fai"),
                "reference-fasta-index",
                b"chr1\t4\n".as_slice(),
            ),
            (
                dictionary.clone(),
                "reference-sequence-dictionary",
                b"@SQ\tSN:chr1\tLN:4\n".as_slice(),
            ),
        ],
    )?;
    let catalog = catalog_root.join("grch38-reference.json");
    write_json(
        &catalog,
        &json!({
            "schema_version": "bijux.data_catalog.v1",
            "id": "grch38-reference",
            "files": catalog_files(&study_root, &files, false),
        }),
    )?;
    let record = index_record(&study_root, "grch38-reference", &catalog, &files)?;
    write_json(&data_index, &governed_data_index(vec![record]))?;
    Ok(BenchmarkReferenceFixture { _temp: temp, study_root, data_index, dictionary })
}

fn bam_genotyping_corpus_fixture() -> Result<BamGenotypingCorpusFixture> {
    let temp = tempfile::tempdir()?;
    let study_root = temp.path().join("STUDY");
    let data_index = study_root.join("data/evidence/indexes/source/data-index.json");
    fs::create_dir_all(data_index.parent().expect("index parent"))?;

    let bam_root = study_root.join("data/corpora/ena-prjeb22390-err2112767-bam");
    let reference_root = study_root.join("data/references/equcab3-gcf-002863925.1");
    let database_root =
        study_root.join("data/databases/ensembl-113-equcab3-refseq-known-sites-10mb");
    let catalog_root = study_root.join("data/catalog");
    fs::create_dir_all(&bam_root)?;
    fs::create_dir_all(&reference_root)?;
    fs::create_dir_all(&database_root)?;
    fs::create_dir_all(&catalog_root)?;

    let bam_files = write_authority_files(
        &study_root,
        &[
            (
                bam_root.join("ERR2112767.sorted.bam"),
                "coordinate-sorted-deduplicated-bam",
                b"BAM-bytes".as_slice(),
            ),
            (bam_root.join("ERR2112767.sorted.bam.bai"), "bam-index", b"index".as_slice()),
        ],
    )?;
    let reference_files = write_authority_files(
        &study_root,
        &[
            (reference_root.join("EquCab3.0.fna"), "catalog.files.0", b">chr1\nAC\n".as_slice()),
            (reference_root.join("EquCab3.0.fna.fai"), "catalog.files.1", b"chr1\t2\n".as_slice()),
        ],
    )?;
    let known_sites_files = write_authority_files(
        &study_root,
        &[
            (
                database_root.join("known-sites.vcf.gz"),
                "normalized-known-sites-vcf",
                b"compressed-vcf".as_slice(),
            ),
            (
                database_root.join("known-sites.vcf.gz.csi"),
                "known-sites-vcf-index",
                b"csi".as_slice(),
            ),
            (
                database_root.join("known-sites.bed"),
                "zero-based-candidate-regions",
                b"chr1\t0\t2\n".as_slice(),
            ),
        ],
    )?;
    let allele_files = write_authority_files(
        &study_root,
        &[(
            database_root.join("known-sites.alleles.tsv"),
            "bcftools-constrained-calling-alleles",
            b"chr1\t1\tA,C\n".as_slice(),
        )],
    )?;

    let bam_catalog = catalog_root.join("ena-prjeb22390-err2112767-bam.json");
    write_json(
        &bam_catalog,
        &json!({
            "schema_version": "bijux.data_catalog.v1",
            "id": "ena-prjeb22390-err2112767-bam",
            "run_accession": "ERR2112767",
            "files": catalog_files(&study_root, &bam_files, false),
        }),
    )?;
    let reference_catalog = catalog_root.join("equcab3-gcf-002863925.1.json");
    write_json(
        &reference_catalog,
        &json!({
            "schema_version": "bijux.data_catalog.v1",
            "id": "equcab3-gcf-002863925.1",
            "files": catalog_files(&study_root, &reference_files, true),
        }),
    )?;
    let known_sites_catalog = catalog_root.join("ensembl-113-equcab3-refseq-known-sites-10mb.json");
    write_json(
        &known_sites_catalog,
        &json!({
            "schema_version": "bijux.data_catalog.v1",
            "id": "ensembl-113-equcab3-refseq-known-sites-10mb",
            "files": catalog_files(&study_root, &known_sites_files, false),
        }),
    )?;
    let alleles_catalog =
        catalog_root.join("ensembl-113-equcab3-refseq-genotyping-inputs-10mb.json");
    write_json(
        &alleles_catalog,
        &json!({
            "schema_version": "bijux.data_catalog.v1",
            "id": "ensembl-113-equcab3-refseq-genotyping-inputs-10mb",
            "files": catalog_files(&study_root, &allele_files, false),
        }),
    )?;

    let records = vec![
        index_record(&study_root, "ena-prjeb22390-err2112767-bam", &bam_catalog, &bam_files)?,
        index_record(&study_root, "equcab3-gcf-002863925.1", &reference_catalog, &reference_files)?,
        index_record(
            &study_root,
            "ensembl-113-equcab3-refseq-known-sites-10mb",
            &known_sites_catalog,
            &known_sites_files,
        )?,
        index_record(
            &study_root,
            "ensembl-113-equcab3-refseq-genotyping-inputs-10mb",
            &alleles_catalog,
            &allele_files,
        )?,
    ];
    write_json(&data_index, &governed_data_index(records))?;
    Ok(BamGenotypingCorpusFixture { _temp: temp, study_root, data_index })
}

fn write_authority_files(
    study_root: &Path,
    files: &[(PathBuf, &str, &[u8])],
) -> Result<Vec<Value>> {
    files
        .iter()
        .map(|(path, role, bytes)| {
            fs::write(path, bytes)?;
            Ok(json!({
                "path": format!("study://{}", path.strip_prefix(study_root)?.display()),
                "role": role,
                "sha256": bijux_dna_infra::hash_bytes_sha256(bytes),
                "size_bytes": bytes.len(),
                "legacy_fields": ["path", "sha256", "size_bytes"],
            }))
        })
        .collect()
}

fn catalog_files(study_root: &Path, files: &[Value], governed: bool) -> Vec<Value> {
    files
        .iter()
        .map(|file| {
            let relative = file["path"].as_str().expect("file path").trim_start_matches("study://");
            let path = study_root.join(relative);
            if governed {
                json!({
                    "governed_path": path,
                    "governed_sha256": file["sha256"],
                    "governed_size_bytes": file["size_bytes"],
                })
            } else {
                json!({
                    "path": path,
                    "kind": file["role"],
                    "sha256": file["sha256"],
                    "size_bytes": file["size_bytes"],
                })
            }
        })
        .collect()
}

fn index_record(study_root: &Path, id: &str, catalog: &Path, files: &[Value]) -> Result<Value> {
    Ok(json!({
        "id": id,
        "benchmark_eligibility": "eligible",
        "integrity_status": "verified",
        "catalog_path": format!("study://{}", catalog.strip_prefix(study_root)?.display()),
        "catalog_sha256": bijux_dna_infra::hash_file_sha256(catalog)?,
        "purpose": "exercise governed genotyping admission",
        "source_identities": ["source:test-fixture"],
        "access_classification": "test-only",
        "redistribution_classification": "test-only",
        "scientific_name": "Equus caballus",
        "reference_assembly": "EquCab3.0",
        "temporal_provenance": {
            "field": "created_at_utc",
            "timestamp": "2026-08-05T00:00:00Z"
        },
        "files": files,
        "known_limitations": ["synthetic test data"],
        "missing_provenance": [],
    }))
}

fn governed_data_index(mut records: Vec<Value>) -> Value {
    records.sort_by(|left, right| {
        left["id"].as_str().expect("record id").cmp(right["id"].as_str().expect("record id"))
    });
    for record in &mut records {
        record["files"].as_array_mut().expect("record files").sort_by(|left, right| {
            left["path"]
                .as_str()
                .expect("file path")
                .cmp(right["path"].as_str().expect("file path"))
        });
    }
    let file_count =
        records.iter().map(|record| record["files"].as_array().unwrap().len()).sum::<usize>();
    let size_bytes = records
        .iter()
        .flat_map(|record| record["files"].as_array().expect("record files"))
        .map(|file| file["size_bytes"].as_u64().expect("file size"))
        .sum::<u64>();
    json!({
        "schema_version": "bijux.data_index.v1",
        "created_at_utc": "2026-08-05T00:00:00Z",
        "data_root": "study://data",
        "summary": {
            "record_count": records.len(),
            "file_count": file_count,
            "size_bytes": size_bytes,
            "integrity_verified_count": records.len(),
            "benchmark_eligibility": {"eligible": records.len()},
        },
        "records": records,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}
