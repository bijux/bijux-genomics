use std::fs;
use std::io::Write as _;

use anyhow::Context as _;
use bijux_dna_bench::{
    materialize_fastq_corpus, BenchmarkCorpusClass, CorpusAcquisitionFile, CorpusAcquisitionLayout,
    CorpusAcquisitionPlan, CorpusAcquisitionSample, CorpusEvaluationDesign,
    CorpusMaterializationDisposition, CorpusMaterializationRequest, CorpusSourcePolicy,
    CORPUS_ACQUISITION_SCHEMA_V2,
};
use flate2::write::GzEncoder;
use flate2::Compression;

const REPOSITORY_COMMIT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn catalog_snapshot_path(data_root: &std::path::Path) -> std::path::PathBuf {
    data_root.join("catalog/snapshots").join(REPOSITORY_COMMIT).join("corpus-01.json")
}

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
    let target = data_root.join("corpora/corpus-01/raw/ERR000001/ERR000001.fastq.gz");
    fs::create_dir_all(target.parent().context("FASTQ parent")?)?;
    fs::write(&target, &fastq_bytes)?;

    let publication = materialize_fastq_corpus(&CorpusMaterializationRequest {
        plan_path,
        data_root: data_root.clone(),
        repository_commit: REPOSITORY_COMMIT.to_string(),
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
fn selected_collections_in_one_portfolio_receive_independent_catalogs() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let data_root = temporary.path().canonicalize()?.join("data");
    fs::create_dir(&data_root)?;
    let data_root = data_root.canonicalize()?;
    let fastq_bytes = compressed_fastq()?;

    let first = acquisition_plan(&fastq_bytes)?;
    let mut second = first.clone();
    second.corpus_id = "corpus-02".to_string();
    second.samples[0].accession = "ERR000002".to_string();
    second.samples[0].sample_accession = "SAMEA2".to_string();
    second.samples[0].experiment_accession = "ERX2".to_string();
    second.samples[0].files[0].name = "ERR000002.fastq.gz".to_string();
    second.samples[0].files[0].source_url =
        "https://archive.example/ERR000002.fastq.gz".to_string();

    let plan_root = data_root.join("evidence/acquisition-plans");
    fs::create_dir_all(&plan_root)?;
    let first_plan = plan_root.join("corpus-01.json");
    let second_plan = plan_root.join("corpus-02.json");
    fs::write(&first_plan, serde_json::to_vec_pretty(&first)?)?;
    fs::write(&second_plan, serde_json::to_vec_pretty(&second)?)?;
    let first_fastq = data_root.join("corpora/corpus-01/raw/ERR000001/ERR000001.fastq.gz");
    let second_fastq = data_root.join("corpora/corpus-02/raw/ERR000002/ERR000002.fastq.gz");
    fs::create_dir_all(first_fastq.parent().context("first FASTQ parent")?)?;
    fs::create_dir_all(second_fastq.parent().context("second FASTQ parent")?)?;
    fs::write(first_fastq, &fastq_bytes)?;
    fs::write(second_fastq, &fastq_bytes)?;

    let first_publication = materialize_fastq_corpus(&request(first_plan, data_root.clone()))?;
    let second_publication = materialize_fastq_corpus(&request(second_plan, data_root.clone()))?;

    assert_eq!(first_publication.catalog.id, "corpus-01");
    assert_eq!(second_publication.catalog.id, "corpus-02");
    assert_ne!(first_publication.catalog_path, second_publication.catalog_path);
    assert!(first_publication.catalog_path.is_file());
    assert!(second_publication.catalog_path.is_file());
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
    assert!(!catalog_snapshot_path(&data_root).exists());
    Ok(())
}

#[test]
fn legacy_flat_catalog_is_preserved_while_snapshot_is_published() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let data_root = temporary.path().canonicalize()?.join("data");
    fs::create_dir(&data_root)?;
    let data_root = data_root.canonicalize()?;
    let fastq_bytes = compressed_fastq()?;
    let plan_path = write_plan_and_fastq(&data_root, &fastq_bytes)?;
    let legacy_catalog_path = data_root.join("catalog/human-short-read-diversity.json");
    fs::create_dir_all(legacy_catalog_path.parent().context("catalog parent")?)?;
    fs::write(&legacy_catalog_path, b"legacy v1 authority")?;

    let publication = materialize_fastq_corpus(&request(plan_path, data_root.clone()))?;

    assert_eq!(publication.catalog_path, catalog_snapshot_path(&data_root));
    assert_eq!(fs::read(legacy_catalog_path)?, b"legacy v1 authority");
    Ok(())
}

#[test]
fn existing_valid_catalog_is_revalidated_without_mutation() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let data_root = temporary.path().canonicalize()?.join("data");
    fs::create_dir(&data_root)?;
    let data_root = data_root.canonicalize()?;
    let fastq_bytes = compressed_fastq()?;
    let plan_path = write_plan_and_fastq(&data_root, &fastq_bytes)?;
    let initial = materialize_fastq_corpus(&request(plan_path.clone(), data_root.clone()))?;
    let mut enriched = initial.catalog.clone();
    enriched.retrieved_at_utc = "2026-08-05T23:00:00+00:00".to_string();
    fs::write(&initial.catalog_path, serde_json::to_vec_pretty(&enriched)?)?;
    let catalog_bytes = fs::read(&initial.catalog_path)?;

    let mut replay = request(plan_path, data_root);
    replay.admitted_at_utc = "2026-08-06T02:00:00+00:00".to_string();
    let publication = materialize_fastq_corpus(&replay)?;

    assert_eq!(publication.downloaded_files, 0);
    assert_eq!(publication.verified_existing_files, 1);
    assert_eq!(publication.catalog, enriched);
    assert_eq!(fs::read(publication.catalog_path)?, catalog_bytes);
    Ok(())
}

#[test]
fn existing_catalog_source_drift_is_preserved_and_rejected() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let data_root = temporary.path().canonicalize()?.join("data");
    fs::create_dir(&data_root)?;
    let data_root = data_root.canonicalize()?;
    let fastq_bytes = compressed_fastq()?;
    let plan_path = write_plan_and_fastq(&data_root, &fastq_bytes)?;
    let publication = materialize_fastq_corpus(&request(plan_path.clone(), data_root.clone()))?;
    let mut drifted = publication.catalog;
    drifted.files[0].source_url = "https://archive.example/drift.fastq.gz".to_string();
    let drifted_bytes = serde_json::to_vec_pretty(&drifted)?;
    fs::write(&publication.catalog_path, &drifted_bytes)?;

    let Err(error) = materialize_fastq_corpus(&request(plan_path, data_root)) else {
        anyhow::bail!("catalog source drift must fail replay admission");
    };

    assert!(format!("{error:#}").contains("differs from acquisition plan or FASTQ"));
    assert_eq!(fs::read(publication.catalog_path)?, drifted_bytes);
    Ok(())
}

#[test]
fn existing_catalog_policy_drift_is_preserved_and_rejected() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let data_root = temporary.path().canonicalize()?.join("data");
    fs::create_dir(&data_root)?;
    let data_root = data_root.canonicalize()?;
    let fastq_bytes = compressed_fastq()?;
    let plan_path = write_plan_and_fastq(&data_root, &fastq_bytes)?;
    let publication = materialize_fastq_corpus(&request(plan_path.clone(), data_root.clone()))?;
    let mut drifted = publication.catalog;
    drifted.policy.attribution = "unbound attribution".to_string();
    let drifted_bytes = serde_json::to_vec_pretty(&drifted)?;
    fs::write(&publication.catalog_path, &drifted_bytes)?;

    let Err(error) = materialize_fastq_corpus(&request(plan_path, data_root)) else {
        anyhow::bail!("catalog policy drift must fail replay admission");
    };

    assert!(format!("{error:#}").contains("differs from acquisition plan identity"));
    assert_eq!(fs::read(publication.catalog_path)?, drifted_bytes);
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
fn materialization_rejects_invalid_repository_commit_before_io() {
    let mut request = request("/missing/plan.json".into(), "/missing/data".into());
    request.repository_commit = "A".repeat(40);

    let Err(error) = materialize_fastq_corpus(&request) else {
        panic!("invalid repository commit must fail");
    };

    assert!(error.to_string().contains("lowercase 40-character Git object id"));
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
    assert!(!catalog_snapshot_path(&data_root).exists());
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
    let target = data_root.join("corpora/corpus-01/raw/ERR000001/ERR000001.fastq.gz");
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
        repository_commit: REPOSITORY_COMMIT.to_string(),
        maximum_parallelism: 1,
        retries: 0,
        timeout_seconds: 10,
        admitted_at_utc: "2026-08-06T01:00:00+00:00".to_string(),
    }
}

fn acquisition_plan(fastq: &[u8]) -> anyhow::Result<CorpusAcquisitionPlan> {
    Ok(CorpusAcquisitionPlan {
        schema_version: CORPUS_ACQUISITION_SCHEMA_V2.to_string(),
        corpus_id: "corpus-01".to_string(),
        portfolio_id: "human-short-read-diversity".to_string(),
        species: "Homo sapiens".to_string(),
        retrieved_at_utc: "2026-08-06T00:00:00+00:00".to_string(),
        source_authority: "European Nucleotide Archive".to_string(),
        description: "real human short-read diversity corpus for comparative FASTQ campaigns"
            .to_string(),
        policy: CorpusSourcePolicy {
            class: BenchmarkCorpusClass::ProductionReal,
            access_classification: "public".to_string(),
            redistribution_classification: "public-source-copy".to_string(),
            access_terms_url: "https://www.ebi.ac.uk/ena/browser/about/policies".to_string(),
            attribution: "cite source studies and accessions".to_string(),
            projected_storage_bytes: u64::try_from(fastq.len())?,
            projected_cpu_hours: 1,
            projection_basis: "bounded contract-test projection".to_string(),
            truth_roles: vec!["archive_context".to_string()],
            truth_strategy: "observational_only".to_string(),
            known_limitations: vec!["not population representative".to_string()],
        },
        evaluation: CorpusEvaluationDesign {
            sample_context: "One public short-read library.".to_string(),
            assay_context: "Illumina whole-genome sequencing.".to_string(),
            strata: vec!["era".to_string(), "layout".to_string()],
            controls: vec!["contract_validation".to_string()],
            held_out_split: "No scientific split; materialization contract use only.".to_string(),
            lineage: "Acquisition plan to admitted catalog.".to_string(),
            allowed_uses: vec!["contract_validation".to_string()],
        },
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
