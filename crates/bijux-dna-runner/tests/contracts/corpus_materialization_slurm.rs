use std::fs;

use bijux_dna_bench_model::{
    BenchmarkCorpusClass, CorpusAcquisitionFile, CorpusAcquisitionLayout, CorpusAcquisitionPlan,
    CorpusAcquisitionSample, CorpusEvaluationDesign, CorpusMaterializationDisposition,
    CorpusSourcePolicy, GovernedDataCatalog, GovernedDataCatalogFile, GovernedDataCatalogSample,
    CORPUS_ACQUISITION_SCHEMA_V2, GOVERNED_DATA_CATALOG_SCHEMA_V2,
};
use bijux_dna_runner::api::{
    collect_corpus_materialization_slurm_job, load_corpus_materialization_slurm_spec,
    load_prepared_corpus_materialization_slurm_job, prepare_corpus_materialization_slurm_job,
    publish_corpus_materialization_evidence_correction, submit_prepared_corpus_materialization_job,
    CorpusMaterializationEvidenceCorrectionRequest, CorpusMaterializationEvidenceStatus,
    CorpusMaterializationSlurmResources, CorpusMaterializationSlurmSpec, SlurmClientPaths,
    CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1,
};

#[cfg(unix)]
use crate::scheduler_executable::write_executable;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt as _;

#[test]
fn preparation_binds_exact_authorities_and_renders_rust_cli() -> anyhow::Result<()> {
    let fixture = fixture()?;

    let prepared = prepare_corpus_materialization_slurm_job(&fixture.spec, "2026-08-06T10:20:00Z")?;
    let script = fs::read_to_string(&prepared.request.script_path)?;

    assert!(script.contains("#SBATCH --cpus-per-task=2"));
    assert!(script.contains("'bench' 'materialize-fastq-corpus'"));
    assert!(script.contains("'--repository-commit' 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'"));
    assert!(script.contains(&fixture.spec.executable_sha256));
    assert!(script.contains(&fixture.spec.plan_sha256));
    assert!(!script.contains("python"));
    assert!(fixture.spec.evidence_root.join("materialization-spec.json").is_file());
    assert!(fixture.spec.evidence_root.join("scheduler/prepared-job.json").is_file());
    assert_eq!(load_prepared_corpus_materialization_slurm_job(&fixture.spec)?, prepared);
    let input_spec = fixture.root.path().join("input-spec.json");
    fs::write(&input_spec, serde_json::to_vec_pretty(&fixture.spec)?)?;
    assert_eq!(load_corpus_materialization_slurm_spec(&input_spec)?, fixture.spec);
    assert_eq!(
        prepared,
        prepare_corpus_materialization_slurm_job(&fixture.spec, "2026-08-06T10:20:00Z")?
    );
    Ok(())
}

#[test]
fn preparation_binds_an_existing_catalog_for_idempotent_replay() -> anyhow::Result<()> {
    let mut fixture = fixture()?;
    publish_catalog(&fixture)?;
    let catalog_path = catalog_snapshot_path(&fixture);
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
    fixture.spec.existing_catalog_sha256 = Some(catalog_sha256.clone());

    let prepared = prepare_corpus_materialization_slurm_job(&fixture.spec, "2026-08-06T10:20:00Z")?;
    let script = fs::read_to_string(&prepared.request.script_path)?;

    assert!(script.contains(&catalog_sha256));
    assert!(script.contains("sha256sum \"${CATALOG}\""));
    assert!(!script.contains("test ! -e \"${CATALOG}\""));
    Ok(())
}

#[test]
fn preparation_rejects_existing_catalog_digest_drift() -> anyhow::Result<()> {
    let mut fixture = fixture()?;
    publish_catalog(&fixture)?;
    fixture.spec.existing_catalog_sha256 = Some("d".repeat(64));

    assert!(prepare_corpus_materialization_slurm_job(&fixture.spec, "2026-08-06T10:20:00Z")
        .is_err_and(|error| error.to_string().contains("digest differs")));
    assert!(!fixture.spec.evidence_root.exists());
    Ok(())
}

#[test]
fn preparation_rejects_scheduler_injection_and_authority_drift() -> anyhow::Result<()> {
    let mut missing_command = fixture()?;
    missing_command.spec.command_arguments.clear();
    assert!(prepare_corpus_materialization_slurm_job(
        &missing_command.spec,
        "2026-08-06T10:20:00Z"
    )
    .is_err_and(|error| error.to_string().contains("command must contain")));

    let mut injected = fixture()?;
    injected.spec.resources.partition = "compute\n#SBATCH --exclusive".to_string();
    assert!(prepare_corpus_materialization_slurm_job(&injected.spec, "2026-08-06T10:20:00Z")
        .is_err_and(|error| error.to_string().contains("unsupported characters")));

    let drifted = fixture()?;
    fs::write(&drifted.spec.plan_path, b"changed")?;
    assert!(prepare_corpus_materialization_slurm_job(&drifted.spec, "2026-08-06T10:20:00Z")
        .is_err_and(|error| error.to_string().contains("digest differs")));
    Ok(())
}

#[test]
fn historical_specifications_remain_loadable_but_cannot_prepare_without_a_command(
) -> anyhow::Result<()> {
    let fixture = fixture()?;
    let mut historical = serde_json::to_value(&fixture.spec)?;
    historical
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("serialized specification is not an object"))?
        .remove("command_arguments");
    let path = fixture.root.path().join("historical-specification.json");
    fs::write(&path, serde_json::to_vec_pretty(&historical)?)?;

    let loaded = load_corpus_materialization_slurm_spec(&path)?;
    assert!(loaded.command_arguments.is_empty());
    assert!(prepare_corpus_materialization_slurm_job(&loaded, "2026-08-06T10:20:00Z")
        .is_err_and(|error| error.to_string().contains("command must contain")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn submission_is_duplicate_safe_and_terminal_collection_verifies_catalog() -> anyhow::Result<()> {
    let fixture = fixture()?;
    prepare_corpus_materialization_slurm_job(&fixture.spec, "2026-08-06T10:20:00Z")?;
    let prepared = load_prepared_corpus_materialization_slurm_job(&fixture.spec)?;
    let calls = fixture.root.path().join("sbatch.calls");
    let scheduler = scheduler_client(
        fixture.root.path(),
        &format!("printf x >> '{}'\nprintf '456789\n'\n", calls.display()),
    )?;

    let first = submit_prepared_corpus_materialization_job(&scheduler, &prepared)?;
    let recovered = submit_prepared_corpus_materialization_job(&scheduler, &prepared)?;
    assert_eq!(first, recovered);
    assert_eq!(fs::read_to_string(&calls)?, "x");

    write_executable(&scheduler.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &scheduler.sacct,
        "#!/bin/sh\nprintf '%s\n' '456789|COMPLETED|0:0|12|00:00:10|128K|n001|None'\n",
    )?;
    fs::write(&prepared.request.stdout_path, b"")?;
    fs::write(&prepared.request.stderr_path, b"")?;
    fs::write(
        fixture.spec.evidence_root.join("scheduler/completed.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.runner.corpus_materialization_completion.v1",
            "specification_sha256": prepared.specification_sha256,
        }))?,
    )?;
    let catalog = publish_catalog(&fixture)?;
    let catalog_path = catalog_snapshot_path(&fixture);
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
    fs::write(
        fixture.spec.evidence_root.join("materialization-publication.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "catalog_path": catalog_path,
            "catalog_sha256": catalog_sha256,
            "downloaded_files": 1,
            "verified_existing_files": 0,
            "catalog": catalog,
        }))?,
    )?;

    let collection = collect_corpus_materialization_slurm_job(
        &scheduler,
        &fixture.spec,
        "2026-08-06T10:30:00Z",
    )?;
    let evidence = collection.evidence.ok_or_else(|| anyhow::anyhow!("missing evidence"))?;

    assert_eq!(evidence.status, CorpusMaterializationEvidenceStatus::Passed);
    assert_eq!(evidence.catalog_sha256.as_deref(), Some(catalog_sha256.as_str()));
    let evidence_path = fixture.spec.evidence_root.join("materialization-evidence.json");
    assert!(evidence_path.is_file());
    let correction_path =
        fixture.spec.evidence_root.join("materialization-evidence-correction.json");
    let correction_request = CorpusMaterializationEvidenceCorrectionRequest {
        evidence_path,
        output_path: correction_path.clone(),
        observed_local_time: "2026-08-06T10:30:00+02:00".to_string(),
        source_timezone: "Europe/Stockholm".to_string(),
        corrected_at_utc: "2026-08-06T11:00:00Z".to_string(),
        source_commit: "b".repeat(40),
        reason: "The operator supplied the scheduler-local clock as a UTC timestamp.".to_string(),
    };
    let correction = publish_corpus_materialization_evidence_correction(&correction_request)?;
    assert_eq!(correction.recorded_value, "2026-08-06T10:30:00Z");
    assert_eq!(correction.corrected_value, "2026-08-06T08:30:00Z");
    assert_eq!(
        correction,
        publish_corpus_materialization_evidence_correction(&correction_request)?
    );
    let mut conflicting_correction = correction_request;
    conflicting_correction.observed_local_time = "2026-08-06T10:31:00+02:00".to_string();
    assert!(publish_corpus_materialization_evidence_correction(&conflicting_correction)
        .is_err_and(|error| error.to_string().contains("conflicts")));
    fs::write(&prepared.request.stdout_path, b"tampered")?;
    assert!(collect_corpus_materialization_slurm_job(
        &scheduler,
        &fixture.spec,
        "2026-08-06T10:31:00Z",
    )
    .is_err_and(|error| error.to_string().contains("scheduler log digests")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn active_collection_returns_without_waiting_or_publishing() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let prepared = prepare_corpus_materialization_slurm_job(&fixture.spec, "2026-08-06T10:20:00Z")?;
    let scheduler = scheduler_client(fixture.root.path(), "printf '456789\n'\n")?;
    submit_prepared_corpus_materialization_job(&scheduler, &prepared)?;
    write_executable(&scheduler.squeue, "#!/bin/sh\nprintf 'RUNNING\\n'\n")?;

    let collection = collect_corpus_materialization_slurm_job(
        &scheduler,
        &fixture.spec,
        "2026-08-06T10:21:00Z",
    )?;

    assert_eq!(collection.observation.state, bijux_dna_runner::api::SlurmJobState::Running);
    assert!(collection.evidence.is_none());
    assert!(!fixture.spec.evidence_root.join("materialization-evidence.json").exists());
    Ok(())
}

struct Fixture {
    root: tempfile::TempDir,
    spec: CorpusMaterializationSlurmSpec,
    plan: CorpusAcquisitionPlan,
    fastq_path: std::path::PathBuf,
}

fn fixture() -> anyhow::Result<Fixture> {
    let root = tempfile::tempdir()?;
    let repository = root.path().join("repository");
    let data_root = root.path().join("study/data");
    fs::create_dir(&repository)?;
    fs::create_dir_all(data_root.join("evidence/acquisition-plans"))?;
    fs::create_dir_all(data_root.join("evidence/materializations"))?;
    let repository = repository.canonicalize()?;
    let data_root = data_root.canonicalize()?;
    let executable = root.path().join("bijux-dna");
    fs::write(&executable, b"executable")?;
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&executable)?.permissions();
        permissions.set_mode(0o750);
        fs::set_permissions(&executable, permissions)?;
    }
    let executable = executable.canonicalize()?;
    let fastq_bytes = b"compressed-fastq".to_vec();
    let plan = acquisition_plan(u64::try_from(fastq_bytes.len())?);
    let plan_path = data_root.join("evidence/acquisition-plans/plan.json");
    fs::write(&plan_path, serde_json::to_vec_pretty(&plan)?)?;
    let plan_path = plan_path.canonicalize()?;
    let fastq_path =
        data_root.join("corpora/human-short-read-diversity/raw/ERR000001/ERR000001.fastq.gz");
    let command_arguments = vec![
        "bench".to_string(),
        "materialize-fastq-corpus".to_string(),
        "--plan".to_string(),
        plan_path.display().to_string(),
        "--data-root".to_string(),
        data_root.display().to_string(),
        "--repository-commit".to_string(),
        "a".repeat(40),
        "--jobs".to_string(),
        "2".to_string(),
        "--retries".to_string(),
        "2".to_string(),
        "--timeout-seconds".to_string(),
        "120".to_string(),
        "--admitted-at-utc".to_string(),
        "2026-08-06T10:25:00Z".to_string(),
        "--json".to_string(),
    ];
    let spec = CorpusMaterializationSlurmSpec {
        schema_version: CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1.to_string(),
        run_id: "run-2042".to_string(),
        attempt_id: "acquisition-attempt-001".to_string(),
        portfolio_id: "human-short-read-diversity".to_string(),
        repository_path: repository,
        repository_commit: "a".repeat(40),
        executable_sha256: bijux_dna_infra::hash_file_sha256(&executable)?,
        executable_path: executable,
        command_arguments,
        plan_sha256: bijux_dna_infra::hash_file_sha256(&plan_path)?,
        plan_path,
        existing_catalog_sha256: None,
        data_root: data_root.clone(),
        evidence_root: data_root.join("evidence/materializations/run-2042"),
        resources: CorpusMaterializationSlurmResources {
            account: "project".to_string(),
            partition: "compute".to_string(),
            cpus: 2,
            memory_mb: 4096,
            walltime: "01:00:00".to_string(),
        },
        maximum_parallelism: 2,
        retries: 2,
        timeout_seconds: 120,
        admitted_at_utc: "2026-08-06T10:25:00Z".to_string(),
    };
    Ok(Fixture { root, spec, plan, fastq_path })
}

fn acquisition_plan(size_bytes: u64) -> CorpusAcquisitionPlan {
    CorpusAcquisitionPlan {
        schema_version: CORPUS_ACQUISITION_SCHEMA_V2.to_string(),
        corpus_id: "corpus-01".to_string(),
        portfolio_id: "human-short-read-diversity".to_string(),
        species: "Homo sapiens".to_string(),
        retrieved_at_utc: "2026-08-06T10:00:00Z".to_string(),
        source_authority: "European Nucleotide Archive".to_string(),
        description: "real benchmark corpus".to_string(),
        policy: source_policy(size_bytes),
        evaluation: evaluation_design(),
        source_spec: "benchmarks/corpora/human-short-read-diversity.toml".to_string(),
        source_spec_sha256: "b".repeat(64),
        sample_count: 1,
        file_count: 1,
        total_size_bytes: size_bytes,
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
                source_md5: "c".repeat(32),
                size_bytes,
            }],
        }],
    }
}

fn publish_catalog(fixture: &Fixture) -> anyhow::Result<GovernedDataCatalog> {
    fs::create_dir_all(
        fixture.fastq_path.parent().ok_or_else(|| anyhow::anyhow!("FASTQ parent"))?,
    )?;
    fs::write(&fixture.fastq_path, b"compressed-fastq")?;
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&fixture.fastq_path)?.permissions();
        permissions.set_mode(0o444);
        fs::set_permissions(&fixture.fastq_path, permissions)?;
    }
    let sample = &fixture.plan.samples[0];
    let catalog_sample = GovernedDataCatalogSample {
        accession: sample.accession.clone(),
        study_accession: sample.study_accession.clone(),
        sample_accession: sample.sample_accession.clone(),
        experiment_accession: sample.experiment_accession.clone(),
        era: sample.era.clone(),
        layout: sample.layout,
        size_band: sample.size_band.clone(),
        selection_reason: sample.selection_reason.clone(),
        scientific_name: sample.scientific_name.clone(),
        library_source: sample.library_source.clone(),
        library_strategy: sample.library_strategy.clone(),
        library_selection: sample.library_selection.clone(),
        instrument_platform: sample.instrument_platform.clone(),
        instrument_model: sample.instrument_model.clone(),
        archive_base_count: sample.archive_base_count,
        archive_read_count: sample.archive_read_count,
        first_public: sample.first_public.clone(),
        last_updated: sample.last_updated.clone(),
    };
    let catalog = GovernedDataCatalog {
        schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V2.to_string(),
        id: fixture.spec.portfolio_id.clone(),
        purpose: "real benchmark corpus".to_string(),
        source: "European Nucleotide Archive".to_string(),
        policy: fixture.plan.policy.clone(),
        evaluation: fixture.plan.evaluation.clone(),
        source_record: "study://data/evidence/acquisition-plans/plan.json".to_string(),
        source_record_sha256: fixture.spec.plan_sha256.clone(),
        retrieved_at_utc: fixture.plan.retrieved_at_utc.clone(),
        admitted_at_utc: fixture.spec.admitted_at_utc.clone(),
        scientific_name: fixture.plan.species.clone(),
        samples: vec![catalog_sample],
        files: vec![GovernedDataCatalogFile {
            path: fixture.fastq_path.display().to_string(),
            sha256: bijux_dna_infra::hash_file_sha256(&fixture.fastq_path)?,
            size_bytes: fs::metadata(&fixture.fastq_path)?.len(),
            kind: "raw-fastq".to_string(),
            accession: sample.accession.clone(),
            study_accession: sample.study_accession.clone(),
            sample_accession: sample.sample_accession.clone(),
            experiment_accession: sample.experiment_accession.clone(),
            era: sample.era.clone(),
            layout: sample.layout,
            mate: None,
            read_count: 2,
            base_count: 10,
            source_url: sample.files[0].source_url.clone(),
            source_md5: sample.files[0].source_md5.clone(),
            materialization_status: CorpusMaterializationDisposition::Downloaded,
        }],
    };
    let catalog_path = catalog_snapshot_path(fixture);
    fs::create_dir_all(catalog_path.parent().ok_or_else(|| anyhow::anyhow!("catalog parent"))?)?;
    fs::write(catalog_path, serde_json::to_vec_pretty(&catalog)?)?;
    Ok(catalog)
}

fn catalog_snapshot_path(fixture: &Fixture) -> std::path::PathBuf {
    fixture
        .spec
        .data_root
        .join("catalog/snapshots")
        .join(&fixture.spec.repository_commit)
        .join(format!("{}.json", fixture.spec.portfolio_id))
}

fn source_policy(projected_storage_bytes: u64) -> CorpusSourcePolicy {
    CorpusSourcePolicy {
        class: BenchmarkCorpusClass::ProductionReal,
        access_classification: "public".to_string(),
        redistribution_classification: "public-source-copy".to_string(),
        access_terms_url: "https://www.ebi.ac.uk/ena/browser/about/policies".to_string(),
        attribution: "cite source studies and accessions".to_string(),
        projected_storage_bytes,
        projected_cpu_hours: 1,
        projection_basis: "bounded scheduler contract projection".to_string(),
        truth_roles: vec!["archive_context".to_string()],
        truth_strategy: "observational_only".to_string(),
        known_limitations: vec!["not population representative".to_string()],
    }
}

fn evaluation_design() -> CorpusEvaluationDesign {
    CorpusEvaluationDesign {
        sample_context: "One public short-read library.".to_string(),
        assay_context: "Illumina whole-genome sequencing.".to_string(),
        strata: vec!["era".to_string(), "layout".to_string()],
        controls: vec!["contract_validation".to_string()],
        held_out_split: "No scientific split; scheduler contract use only.".to_string(),
        lineage: "Acquisition plan to materialization job to admitted catalog.".to_string(),
        allowed_uses: vec!["contract_validation".to_string()],
    }
}

#[cfg(unix)]
fn scheduler_client(root: &std::path::Path, sbatch_body: &str) -> anyhow::Result<SlurmClientPaths> {
    let paths = SlurmClientPaths {
        sbatch: root.join("sbatch"),
        squeue: root.join("squeue"),
        sacct: root.join("sacct"),
        scancel: root.join("scancel"),
    };
    write_executable(&paths.sbatch, &format!("#!/bin/sh\n{sbatch_body}"))?;
    Ok(paths)
}
