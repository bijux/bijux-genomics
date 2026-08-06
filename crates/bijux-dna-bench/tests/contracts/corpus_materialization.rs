use std::fs;
use std::io::Write as _;

use anyhow::Context as _;
use bijux_dna_bench::{
    materialize_fastq_corpus, CorpusAcquisitionFile, CorpusAcquisitionLayout,
    CorpusAcquisitionPlan, CorpusAcquisitionSample, CorpusMaterializationDisposition,
    CorpusMaterializationRequest, CORPUS_ACQUISITION_SCHEMA_V1,
};
use flate2::write::GzEncoder;
use flate2::Compression;

#[test]
fn materialization_reverifies_existing_fastq_and_publishes_catalog() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let data_root = temporary.path().canonicalize()?.join("data");
    fs::create_dir(&data_root)?;
    let data_root = data_root.canonicalize()?;
    let fastq_bytes = compressed_fastq()?;
    let plan = acquisition_plan(&fastq_bytes)?;
    let plan_path = data_root.join("evidence/acquisition-plans/plan.json");
    fs::create_dir_all(plan_path.parent().context("plan parent")?)?;
    fs::write(&plan_path, serde_json::to_vec_pretty(&plan)?)?;
    let target =
        data_root.join("corpora/human-short-read-diversity/raw/ERR000001/ERR000001.fastq.gz");
    fs::create_dir_all(target.parent().context("FASTQ parent")?)?;
    fs::write(&target, &fastq_bytes)?;

    let publication = materialize_fastq_corpus(&CorpusMaterializationRequest {
        plan_path,
        data_root: data_root.clone(),
        maximum_parallelism: 1,
        retries: 0,
        timeout_seconds: 10,
        admitted_at_utc: "2026-08-06T01:00:00+00:00".to_string(),
    })?;

    assert_eq!(publication.downloaded_files, 0);
    assert_eq!(publication.verified_existing_files, 1);
    assert_eq!(publication.catalog.files[0].read_count, 2);
    assert_eq!(publication.catalog.files[0].base_count, 10);
    assert_eq!(
        publication.catalog.files[0].materialization_status,
        CorpusMaterializationDisposition::VerifiedExisting
    );
    assert!(publication.catalog_path.is_file());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(fs::metadata(target)?.permissions().mode() & 0o777, 0o444);
    }
    Ok(())
}

#[test]
fn malformed_fastq_never_receives_catalog_admission() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let data_root = temporary.path().canonicalize()?.join("data");
    fs::create_dir(&data_root)?;
    let data_root = data_root.canonicalize()?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(b"not-a-header\nACGT\n+\nIIII\n")?;
    let fastq_bytes = encoder.finish()?;
    let plan_path = write_plan_and_fastq(&data_root, &fastq_bytes)?;

    let Err(error) = materialize_fastq_corpus(&request(plan_path, data_root.clone())) else {
        anyhow::bail!("malformed FASTQ must not receive admission");
    };

    assert!(format!("{error:#}").contains("invalid FASTQ record"));
    assert!(!data_root.join("catalog/human-short-read-diversity.json").exists());
    Ok(())
}

#[test]
fn existing_catalog_is_preserved_without_materialization() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let data_root = temporary.path().canonicalize()?.join("data");
    fs::create_dir(&data_root)?;
    let data_root = data_root.canonicalize()?;
    let fastq_bytes = compressed_fastq()?;
    let plan_path = write_plan_and_fastq(&data_root, &fastq_bytes)?;
    let catalog_path = data_root.join("catalog/human-short-read-diversity.json");
    fs::create_dir_all(catalog_path.parent().context("catalog parent")?)?;
    fs::write(&catalog_path, b"existing authority")?;

    let Err(error) = materialize_fastq_corpus(&request(plan_path, data_root)) else {
        anyhow::bail!("existing catalog authority must reject replacement");
    };

    assert!(format!("{error:#}").contains("refusing to replace"));
    assert_eq!(fs::read(catalog_path)?, b"existing authority");
    Ok(())
}

#[test]
fn materialization_rejects_non_utc_admission_before_io() {
    let mut request = request("/missing/plan.json".into(), "/missing/data".into());
    request.admitted_at_utc = "2026-08-06T03:00:00+02:00".to_string();

    let Err(error) = materialize_fastq_corpus(&request) else {
        panic!("non-UTC admission timestamp must fail");
    };

    assert!(error.to_string().contains("must use UTC"));
}

#[test]
fn oversized_fastq_line_never_receives_catalog_admission() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let data_root = temporary.path().canonicalize()?.join("data");
    fs::create_dir(&data_root)?;
    let data_root = data_root.canonicalize()?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(b"@read-one\n")?;
    encoder.write_all(&vec![b'A'; 16 * 1024 * 1024 + 1])?;
    encoder.write_all(b"\n+\nI\n")?;
    let fastq_bytes = encoder.finish()?;
    let plan_path = write_plan_and_fastq(&data_root, &fastq_bytes)?;

    let Err(error) = materialize_fastq_corpus(&request(plan_path, data_root.clone())) else {
        anyhow::bail!("oversized FASTQ line must not receive admission");
    };

    assert!(format!("{error:#}").contains("exceeds 16777216 bytes"));
    assert!(!data_root.join("catalog/human-short-read-diversity.json").exists());
    Ok(())
}

fn compressed_fastq() -> anyhow::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(b"@read-one\nACGT\n+\nIIII\n@read-two\nAACCGG\n+\nIIIIII\n")?;
    Ok(encoder.finish()?)
}

fn write_plan_and_fastq(
    data_root: &std::path::Path,
    fastq: &[u8],
) -> anyhow::Result<std::path::PathBuf> {
    let plan_path = data_root.join("evidence/acquisition-plans/plan.json");
    fs::create_dir_all(plan_path.parent().context("plan parent")?)?;
    fs::write(&plan_path, serde_json::to_vec_pretty(&acquisition_plan(fastq)?)?)?;
    let target =
        data_root.join("corpora/human-short-read-diversity/raw/ERR000001/ERR000001.fastq.gz");
    fs::create_dir_all(target.parent().context("FASTQ parent")?)?;
    fs::write(target, fastq)?;
    Ok(plan_path)
}

fn request(
    plan_path: std::path::PathBuf,
    data_root: std::path::PathBuf,
) -> CorpusMaterializationRequest {
    CorpusMaterializationRequest {
        plan_path,
        data_root,
        maximum_parallelism: 1,
        retries: 0,
        timeout_seconds: 10,
        admitted_at_utc: "2026-08-06T01:00:00+00:00".to_string(),
    }
}

fn acquisition_plan(fastq: &[u8]) -> anyhow::Result<CorpusAcquisitionPlan> {
    Ok(CorpusAcquisitionPlan {
        schema_version: CORPUS_ACQUISITION_SCHEMA_V1.to_string(),
        corpus_id: "corpus-01".to_string(),
        portfolio_id: "human-short-read-diversity".to_string(),
        species: "Homo sapiens".to_string(),
        retrieved_at_utc: "2026-08-06T00:00:00+00:00".to_string(),
        source_authority: "European Nucleotide Archive".to_string(),
        source_spec: "benchmarks/corpora/human-short-read-diversity.toml".to_string(),
        source_spec_sha256: "a".repeat(64),
        sample_count: 1,
        file_count: 1,
        total_size_bytes: u64::try_from(fastq.len())?,
        samples: vec![CorpusAcquisitionSample {
            accession: "ERR000001".to_string(),
            study_accession: "PRJEB1".to_string(),
            sample_accession: "SAMEA1".to_string(),
            experiment_accession: "ERX1".to_string(),
            era: "modern".to_string(),
            layout: CorpusAcquisitionLayout::Se,
            size_band: "under-100mb".to_string(),
            selection_reason: "real single-end coverage".to_string(),
            scientific_name: "Homo sapiens".to_string(),
            library_source: "GENOMIC".to_string(),
            library_strategy: "WGS".to_string(),
            library_selection: "RANDOM".to_string(),
            instrument_platform: "ILLUMINA".to_string(),
            instrument_model: "Illumina HiSeq".to_string(),
            archive_base_count: 10,
            archive_read_count: 2,
            first_public: "2020-01-01".to_string(),
            last_updated: "2020-01-02".to_string(),
            source_report_url: "https://www.ebi.ac.uk/ena/portal/api/filereport".to_string(),
            files: vec![CorpusAcquisitionFile {
                mate: None,
                name: "ERR000001.fastq.gz".to_string(),
                source_url: "https://archive.example/ERR000001.fastq.gz".to_string(),
                source_md5: format!("{:x}", md5::compute(fastq)),
                size_bytes: u64::try_from(fastq.len())?,
            }],
        }],
    })
}
