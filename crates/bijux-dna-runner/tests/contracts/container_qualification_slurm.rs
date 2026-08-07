#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context as _, Result};
use bijux_dna_containers::api::{
    ContainerGovernance, ContainerPathPolicy, ContainerPreflightPolicy, ContainerRuntimePolicy,
    ContainerSitePolicy, ContainerSlurmPolicy,
};
use bijux_dna_runner::api::{
    cancel_container_qualification_slurm_job, collect_container_qualification_slurm_job,
    load_container_qualification_slurm_spec, load_prepared_container_qualification_slurm_job,
    prepare_container_qualification_slurm_job, prepare_historical_container_qualification_replay,
    submit_prepared_container_qualification_job, ContainerQualificationPreparationRequest,
    ContainerQualificationSchedulerStatus, ContainerQualificationSubmissionKind,
    HistoricalContainerQualificationReplayRequest, SlurmClientPaths, SlurmJobState,
};

#[test]
fn preparation_archives_exact_contracts_and_renders_frozen_execution() -> Result<()> {
    let fixture = fixture()?;

    let prepared =
        prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let script = fs::read_to_string(&prepared.request.script_path)?;
    let spec_path = fixture.evidence_root.join("qualification-spec.json");
    let spec = load_container_qualification_slurm_spec(&spec_path)?;

    assert_eq!(spec.schema_version, "bijux.runner.container_qualification_slurm_spec.v2");
    assert!(script.contains("#SBATCH --job-name=fastq-align_reads-bwa-sample-a"));
    assert!(script.contains("#SBATCH --cpus-per-task=2"));
    assert!(script.contains("'execute' '--governance'"));
    assert!(script.contains(&spec.executor_script_sha256));
    assert_eq!(prepared.request.dependencies, vec!["456700"]);
    assert_eq!(spec.repository_path, fixture.frozen_repository);
    assert_eq!(spec.repository_commit, fixture.commit);
    assert!(fixture.evidence_root.join("config-governance.yml").is_file());
    assert!(fixture.evidence_root.join("config-pipeline.yml").is_file());
    assert!(fixture.evidence_root.join("config-credential.yml").is_file());
    assert_eq!(load_prepared_container_qualification_slurm_job(&spec)?, prepared);
    assert_eq!(
        prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?,
        prepared
    );
    Ok(())
}

#[test]
fn preparation_rejects_active_repository_and_archived_contract_conflicts() -> Result<()> {
    let fixture = fixture()?;
    let active_pipeline = fixture.root.path().join("active-pipeline.yml");
    write_pipeline(&active_pipeline, &fixture.active_repository, &fixture.commit, &fixture)?;
    let mut request = fixture.request.clone();
    request.pipeline_path = active_pipeline;
    assert!(prepare_container_qualification_slurm_job(&request, "2026-08-07T09:50:00Z")
        .is_err_and(|error| error.to_string().contains("frozen repository")));

    prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    fs::write(fixture.evidence_root.join("config-credential.yml"), b"conflict\n")?;
    assert!(prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")
        .is_err_and(|error| error.to_string().contains("immutable qualification material")));
    Ok(())
}

#[test]
fn historical_replay_archives_source_authority_without_weakening_current_preparation() -> Result<()>
{
    let fixture = fixture()?;
    let source_pipeline = fixture.root.path().join("source-legacy-pipeline.yml");
    let replay_pipeline = fixture.root.path().join("replay-legacy-pipeline.yml");
    write_legacy_pipeline(
        &source_pipeline,
        &fixture.root.path().join("retired-repository"),
        &"a".repeat(40),
        &fixture.root.path().join("retired-bwa.sif"),
        &"b".repeat(64),
        &fixture,
    )?;
    write_legacy_pipeline(
        &replay_pipeline,
        &fixture.frozen_repository,
        &fixture.commit,
        &fixture.paths.image,
        &fixture.paths.image_sha256,
        &fixture,
    )?;
    let replay_request = HistoricalContainerQualificationReplayRequest {
        hostname: "test-node".to_owned(),
        governance_path: fixture.request.governance_path.clone(),
        source_pipeline_path: source_pipeline.canonicalize()?,
        replay_pipeline_path: replay_pipeline.canonicalize()?,
        credential_reference_path: fixture.request.credential_reference_path.clone(),
        executor_program_path: fixture.request.executor_program_path.clone(),
    };
    let prepared =
        prepare_historical_container_qualification_replay(&replay_request, "2026-08-07T09:50:00Z")?;
    let spec = load_container_qualification_slurm_spec(
        &fixture.evidence_root.join("qualification-spec.json"),
    )?;
    assert_eq!(spec.submission_kind, ContainerQualificationSubmissionKind::HistoricalReplay);
    assert_eq!(spec.schema_version, "bijux.runner.container_qualification_slurm_spec.v2");
    assert!(spec
        .historical_replay_source
        .as_ref()
        .is_some_and(|source| source.pipeline_path.ends_with("source-config-pipeline.yml")));
    assert_eq!(load_prepared_container_qualification_slurm_job(&spec)?, prepared);

    let mut ordinary = fixture.request.clone();
    ordinary.pipeline_path = replay_request.replay_pipeline_path;
    assert!(prepare_container_qualification_slurm_job(&ordinary, "2026-08-07T09:50:00Z")
        .is_err_and(|error| error.to_string().contains("current pipeline schema")));
    Ok(())
}

#[test]
fn lifecycle_is_duplicate_safe_non_waiting_collectable_and_cancellable() -> Result<()> {
    let fixture = fixture()?;
    let prepared =
        prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let spec = load_container_qualification_slurm_spec(
        &fixture.evidence_root.join("qualification-spec.json"),
    )?;
    let calls = fixture.root.path().join("sbatch.calls");
    let scheduler = scheduler_client(
        fixture.root.path(),
        &format!("printf x >> '{}'\nprintf '456789\n'\n", calls.display()),
    )?;

    let first = submit_prepared_container_qualification_job(&scheduler, &prepared)?;
    let recovered = submit_prepared_container_qualification_job(&scheduler, &prepared)?;
    assert_eq!(first, recovered);
    assert_eq!(fs::read_to_string(&calls)?, "x");

    write_executable(&scheduler.squeue, "#!/bin/sh\nprintf 'RUNNING\n'\n")?;
    let active =
        collect_container_qualification_slurm_job(&scheduler, &spec, "2026-08-07T09:51:00Z")?;
    assert_eq!(active.observation.state, SlurmJobState::Running);
    assert!(active.evidence.is_none());
    assert!(!fixture.evidence_root.join("scheduler-evidence.json").exists());

    write_executable(&scheduler.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &scheduler.sacct,
        "#!/bin/sh\nprintf '%s\n' '456789|COMPLETED|0:0|12|00:00:10|128K|n001|None'\n",
    )?;
    fs::write(&prepared.request.stdout_path, b"qualification output\n")?;
    fs::write(&prepared.request.stderr_path, b"")?;
    fs::create_dir_all(&spec.result_root)?;
    fs::write(
        spec.result_root.join("evidence.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "run_id": spec.run_id,
            "status": "passed",
            "evidence_kind": "qualification",
            "repository_commit": spec.repository_commit,
            "repository_path": spec.repository_path,
            "governance_sha256": spec.governance_sha256,
            "pipeline_sha256": spec.pipeline_sha256,
            "credential_reference_sha256": spec.credential_reference_sha256,
        }))?,
    )?;
    let terminal =
        collect_container_qualification_slurm_job(&scheduler, &spec, "2026-08-07T09:52:00Z")?;
    let evidence = terminal.evidence.context("missing terminal evidence")?;
    assert_eq!(evidence.status, ContainerQualificationSchedulerStatus::Passed);
    assert!(evidence.execution_evidence_sha256.is_some());
    assert!(spec.result_root.join("slurm-accounting.json").is_file());
    assert!(spec.result_root.join("qualification-scheduler-evidence.json").is_file());

    let cancellation = fixture.root.path().join("cancelled-job");
    write_executable(
        &scheduler.scancel,
        &format!("#!/bin/sh\nprintf '%s' \"$1\" > '{}'\n", cancellation.display()),
    )?;
    cancel_container_qualification_slurm_job(&scheduler, &spec)?;
    assert_eq!(fs::read_to_string(cancellation)?, "456789");

    fs::write(&prepared.request.stdout_path, b"tampered\n")?;
    assert!(collect_container_qualification_slurm_job(&scheduler, &spec, "2026-08-07T09:53:00Z")
        .is_err_and(|error| error.to_string().contains("scheduler logs")));
    Ok(())
}

#[test]
fn completed_scheduler_job_without_execution_evidence_is_durably_failed() -> Result<()> {
    let fixture = fixture()?;
    let prepared =
        prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let spec = load_container_qualification_slurm_spec(
        &fixture.evidence_root.join("qualification-spec.json"),
    )?;
    let scheduler = scheduler_client(fixture.root.path(), "printf '456789\n'\n")?;
    submit_prepared_container_qualification_job(&scheduler, &prepared)?;
    write_executable(&scheduler.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &scheduler.sacct,
        "#!/bin/sh\nprintf '%s\n' '456789|COMPLETED|0:0|12|00:00:10|128K|n001|None'\n",
    )?;
    fs::write(&prepared.request.stdout_path, b"")?;
    fs::write(&prepared.request.stderr_path, b"execution stopped\n")?;

    let collection =
        collect_container_qualification_slurm_job(&scheduler, &spec, "2026-08-07T09:52:00Z")?;
    let evidence = collection.evidence.context("missing failed terminal evidence")?;

    assert_eq!(evidence.status, ContainerQualificationSchedulerStatus::Failed);
    assert!(evidence
        .failure
        .as_deref()
        .is_some_and(|failure| failure.contains("lacks execution evidence")));
    assert!(spec.result_root.join("slurm-accounting.json").is_file());
    write_executable(&scheduler.squeue, "#!/bin/sh\nexit 99\n")?;
    assert!(collect_container_qualification_slurm_job(&scheduler, &spec, "2026-08-07T09:53:00Z")?
        .evidence
        .is_some());
    Ok(())
}

#[derive(Clone)]
struct FixturePaths {
    image: PathBuf,
    image_sha256: String,
    input: PathBuf,
    input_sha256: String,
    upstream: PathBuf,
    upstream_sha256: String,
    result_root: PathBuf,
}

struct Fixture {
    root: tempfile::TempDir,
    active_repository: PathBuf,
    frozen_repository: PathBuf,
    commit: String,
    request: ContainerQualificationPreparationRequest,
    evidence_root: PathBuf,
    paths: FixturePaths,
}

fn fixture() -> Result<Fixture> {
    let root = tempfile::tempdir()?;
    let study = root.path().join("study");
    let containers = study.join("containers");
    let data = study.join("data");
    let results = study.join("results");
    let finalization = study.join("finalization");
    let active_repository = root.path().join("active-repository");
    let frozen_root = root.path().join("frozen");
    for path in [
        containers.join("apptainer"),
        containers.join("evidence/submissions"),
        data.clone(),
        results.clone(),
        finalization.clone(),
        active_repository.join("containers/contracts"),
        frozen_root.clone(),
        study.join("scratch"),
    ] {
        fs::create_dir_all(path)?;
    }
    let study = study.canonicalize()?;
    let containers = containers.canonicalize()?;
    let data = data.canonicalize()?;
    let results = results.canonicalize()?;
    let finalization = finalization.canonicalize()?;
    let active_repository = active_repository.canonicalize()?;
    let frozen_root = frozen_root.canonicalize()?;
    let (commit, frozen_repository) = setup_repositories(&active_repository, &frozen_root)?;

    let image = containers.join("apptainer/bwa.sif");
    fs::write(&image, b"immutable image")?;
    let input = data.join("reads.fastq.gz");
    fs::write(&input, b"immutable reads")?;
    let upstream = results.join("upstream/result.json");
    fs::create_dir_all(upstream.parent().context("upstream parent")?)?;
    fs::write(&upstream, b"immutable upstream result")?;
    let paths = FixturePaths {
        image_sha256: bijux_dna_infra::hash_file_sha256(&image)?,
        image,
        input_sha256: bijux_dna_infra::hash_file_sha256(&input)?,
        input,
        upstream_sha256: bijux_dna_infra::hash_file_sha256(&upstream)?,
        upstream,
        result_root: results.join("container-validation/run-1200-bwa"),
    };
    let governance = governance(
        ContainerPathPolicy {
            study_root: study.clone(),
            containers_root: containers.clone(),
            data_root: data,
            results_root: results,
            finalization_root: finalization,
            active_repository: active_repository.clone(),
            frozen_repositories: frozen_root,
        },
        &study,
    );
    let governance_path = root.path().join("governance.yml");
    fs::write(&governance_path, serde_json::to_vec_pretty(&governance)?)?;
    let pipeline_path = root.path().join("pipeline.yml");
    let fixture_stub = Fixture {
        root,
        active_repository,
        frozen_repository,
        commit,
        request: ContainerQualificationPreparationRequest {
            hostname: String::new(),
            governance_path: PathBuf::new(),
            pipeline_path: PathBuf::new(),
            credential_reference_path: PathBuf::new(),
            executor_program_path: PathBuf::new(),
        },
        evidence_root: containers.join("evidence/submissions/run-1200-bwa"),
        paths,
    };
    write_pipeline(
        &pipeline_path,
        &fixture_stub.frozen_repository,
        &fixture_stub.commit,
        &fixture_stub,
    )?;
    let credential_reference_path = fixture_stub.root.path().join("credential.yml");
    fs::write(
        &credential_reference_path,
        "schema_version: bijux.container.credential_reference.v1\ncredentials_required: false\nreferences: []\nredaction:\n  publish_values: false\n",
    )?;
    let executor_program_path = fixture_stub.root.path().join("python3");
    write_executable(&executor_program_path, "#!/bin/sh\nexit 0\n")?;
    Ok(Fixture {
        request: ContainerQualificationPreparationRequest {
            hostname: "test-node".to_string(),
            governance_path,
            pipeline_path,
            credential_reference_path,
            executor_program_path,
        },
        ..fixture_stub
    })
}

fn setup_repositories(active_repository: &Path, frozen_root: &Path) -> Result<(String, PathBuf)> {
    git(active_repository, &["init", "--quiet"])?;
    fs::write(
        active_repository.join("containers/contracts/container_validation.py"),
        b"#!/usr/bin/env python3\n",
    )?;
    fs::write(active_repository.join("README.md"), b"governed repository\n")?;
    git(active_repository, &["add", "README.md", "containers/contracts/container_validation.py"])?;
    git(
        active_repository,
        &[
            "-c",
            "user.name=Bijux Test",
            "-c",
            "user.email=bijux@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "initial",
        ],
    )?;
    let commit = git(active_repository, &["rev-parse", "HEAD"])?;
    let frozen_repository = frozen_root.join(&commit);
    let output = Command::new("git")
        .args(["clone", "--quiet"])
        .arg(active_repository)
        .arg(&frozen_repository)
        .output()?;
    ensure!(output.status.success(), "git clone failed");
    Ok((commit, frozen_repository))
}

fn governance(paths: ContainerPathPolicy, study: &Path) -> ContainerGovernance {
    ContainerGovernance {
        schema_version: "bijux.container.governance.v1".to_string(),
        site: ContainerSitePolicy {
            id: "test-site".to_string(),
            hostname_regex: "test-node".to_string(),
        },
        paths,
        runtime: ContainerRuntimePolicy { apptainer_architecture: "linux/amd64".to_string() },
        slurm: ContainerSlurmPolicy {
            account: "science".to_string(),
            scratch_environment: None,
            scratch_root: Some(study.join("scratch")),
            partitions: vec!["compute".to_string()],
            quota_command: vec!["quota".to_string()],
        },
        preflight: ContainerPreflightPolicy {
            minimum_available_bytes: 1,
            minimum_available_inodes: 1,
            maximum_clock_skew_seconds: 1,
            required_commands: vec!["git".to_string()],
            required_contracts: vec!["pipeline".to_string()],
        },
    }
}

fn write_pipeline(path: &Path, repository: &Path, commit: &str, fixture: &Fixture) -> Result<()> {
    fs::write(
        path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.container.pipeline.v2",
            "run": {
                "id": "run-1200",
                "kind": "qualification",
                "campaign": "governed-runner-test",
                "corpus": "immutable-corpus",
                "sample": "sample-a",
                "domain": "fastq",
                "stage": "fastq.align_reads",
                "tool": "bwa",
                "operator": "bijux",
                "submitted_at_utc": "2026-08-07T09:45:00Z"
            },
            "repository": {"path": repository, "commit": commit, "clean": true},
            "image": {"path": fixture.paths.image, "sha256": fixture.paths.image_sha256},
            "data": {"inputs": [
                {
                    "id": "reads",
                    "kind": "data",
                    "path": fixture.paths.input,
                    "mount_path": "/input/reads.fastq.gz",
                    "sha256": fixture.paths.input_sha256
                },
                {
                    "id": "upstream_result",
                    "kind": "upstream_result",
                    "path": fixture.paths.upstream,
                    "mount_path": "/input/upstream.json",
                    "sha256": fixture.paths.upstream_sha256
                }
            ]},
            "commands": {
                "version": ["bwa", "version"],
                "help": ["bwa", "help"],
                "real": ["bwa", "mem"],
                "negative": ["bwa", "invalid"]
            },
            "outputs": {
                "result_root": fixture.paths.result_root,
                "mount_path": "/result",
                "required_files": ["result.json"],
                "collection_rule": "required_files_and_metrics"
            },
            "resources": {
                "account": "science",
                "partition": "compute",
                "cpus": 2,
                "memory_mb": 1024,
                "walltime": "00:05:00",
                "scratch_mb": 1024,
                "retry": {"max_attempts": 1, "backoff_seconds": 0, "retryable_exit_codes": []}
            },
            "execution": {
                "locale": "C",
                "timezone": "UTC",
                "runtime_network": false,
                "threads": 2,
                "random_seed": null,
                "cache_policy": "cold",
                "determinism": "deterministic"
            },
            "dependencies": [{
                "run_id": "run-1199",
                "slurm_job_id": "456700",
                "required_outputs": [fixture.paths.upstream_sha256]
            }],
            "acceptance": {
                "version_regex": "^1[.]0$",
                "negative_exit_nonzero": true,
                "metrics": [{
                    "id": "records",
                    "path": "result.json",
                    "json_pointer": "/records",
                    "operator": "ge",
                    "expected": 1,
                    "unit": "records"
                }]
            }
        }))?,
    )?;
    Ok(())
}

fn write_legacy_pipeline(
    path: &Path,
    repository: &Path,
    commit: &str,
    image: &Path,
    image_sha256: &str,
    fixture: &Fixture,
) -> Result<()> {
    let result_root = fixture
        .paths
        .result_root
        .parent()
        .context("fixture result root has no parent")?
        .join("bwa/run-1200");
    fs::write(
        path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.container.pipeline.v1",
            "run": {
                "id": "run-1200",
                "campaign": "governed-runner-replay-test",
                "corpus": "immutable-corpus",
                "sample": "sample-a",
                "domain": "fastq",
                "stage": "fastq.align_reads",
                "tool": "bwa",
                "operator": "bijux"
            },
            "repository": {"path": repository, "commit": commit, "clean": true},
            "image": {"path": image, "sha256": image_sha256},
            "data": {"inputs": [
                {
                    "id": "reads",
                    "kind": "data",
                    "path": fixture.paths.input,
                    "sha256": fixture.paths.input_sha256
                },
                {
                    "id": "upstream_result",
                    "kind": "upstream_result",
                    "path": fixture.paths.upstream,
                    "sha256": fixture.paths.upstream_sha256
                }
            ]},
            "commands": {
                "version": ["bwa", "version"],
                "help": ["bwa", "help"],
                "real": ["bwa", "mem"],
                "negative": ["bwa", "invalid"]
            },
            "outputs": {"result_root": result_root, "required_files": ["result.json"]},
            "resources": {
                "account": "science",
                "partition": "compute",
                "cpus": 2,
                "memory_mb": 1024,
                "walltime": "00:05:00",
                "scratch_mb": 1024
            },
            "execution": {"locale": "C", "timezone": "UTC", "runtime_network": false},
            "acceptance": {"version_regex": "^1[.]0$", "negative_exit_nonzero": true}
        }))?,
    )?;
    Ok(())
}

fn scheduler_client(root: &Path, sbatch_body: &str) -> Result<SlurmClientPaths> {
    let paths = SlurmClientPaths {
        sbatch: root.join("sbatch"),
        squeue: root.join("squeue"),
        sacct: root.join("sacct"),
        scancel: root.join("scancel"),
    };
    write_executable(&paths.sbatch, sbatch_body)?;
    Ok(paths)
}

fn write_executable(path: &Path, body: &str) -> Result<()> {
    fs::write(path, body)?;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

fn git(repository: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(repository).output()?;
    ensure!(
        output.status.success(),
        "git command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
