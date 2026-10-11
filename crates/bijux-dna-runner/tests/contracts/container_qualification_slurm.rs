#![cfg(unix)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context as _, Result};
use bijux_dna_containers::api::{
    ContainerGovernance, ContainerPathPolicy, ContainerPreflightPolicy, ContainerRuntimePolicy,
    ContainerSitePolicy, ContainerSlurmPolicy,
};
use bijux_dna_runner::api::{
    admit_container_qualification_collection, archive_failed_container_qualification_attempt,
    cancel_container_qualification_slurm_job, collect_container_qualification_slurm_job,
    execute_container_qualification, load_container_qualification_admission,
    load_container_qualification_slurm_spec, load_prepared_container_qualification_slurm_job,
    prepare_container_qualification_slurm_job, prepare_historical_container_qualification_replay,
    rollback_failed_container_qualification_attempt_archive,
    submit_prepared_container_qualification_job, ContainerQualificationExecution,
    ContainerQualificationPreparationRequest, ContainerQualificationSchedulerStatus,
    ContainerQualificationSlurmSpec, ContainerQualificationSubmissionKind,
    HistoricalContainerQualificationReplayRequest, SlurmClientPaths, SlurmJobState,
};

use crate::scheduler_executable::write_executable;

#[test]
fn preparation_archives_exact_contracts_and_renders_frozen_execution() -> Result<()> {
    let fixture = fixture()?;

    let prepared =
        prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let script = fs::read_to_string(&prepared.request.script_path)?;
    let spec_path = fixture.evidence_root.join("qualification-spec.json");
    let spec = load_container_qualification_slurm_spec(&spec_path)?;

    assert_eq!(spec.schema_version, "bijux.runner.container_qualification_slurm_spec.v5");
    assert_eq!(spec.campaign_id.as_deref(), Some("governed-runner-test"));
    assert_eq!(spec.attempt_id.as_deref(), Some("attempt-001"));
    assert!(script.contains("#SBATCH --job-name=fastq-align_reads-bwa-sample-a"));
    assert!(script.contains("#SBATCH --cpus-per-task=2"));
    assert!(script.contains("'containers' 'run' 'qualification-slurm' '--' 'execute'"));
    assert!(!script.contains("container_validation.py"));
    assert_eq!(prepared.request.dependencies, vec!["456700"]);
    assert_eq!(spec.repository_path, fixture.frozen_repository);
    assert_eq!(spec.repository_commit, fixture.commit);
    assert_eq!(spec.result_root, fixture.evidence_root);
    assert!(fixture.evidence_root.join("config-governance.yml").is_file());
    assert!(fixture.evidence_root.join("config-pipeline.yml").is_file());
    assert!(fixture.evidence_root.join("config-credential.yml").is_file());
    assert!(fixture.evidence_root.join("manifests/attempt-layout.json").is_file());
    assert_eq!(load_prepared_container_qualification_slurm_job(&spec)?, prepared);
    assert_eq!(
        prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?,
        prepared
    );
    Ok(())
}

#[test]
fn native_executor_runs_the_admitted_contract_without_a_python_authority() -> Result<()> {
    let fixture = fixture()?;
    prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let spec = load_container_qualification_slurm_spec(
        &fixture.evidence_root.join("qualification-spec.json"),
    )?;
    let execution = execute_with_fake_runtime(
        &fixture,
        &spec,
        r#"#!/bin/sh
set -eu
output_root=
for argument in "$@"; do
  case "$argument" in
    *:/result:rw) output_root=${argument%:/result:rw} ;;
  esac
done
case " $* " in
  *" bwa version "*) printf '1.0\n' ;;
  *" bwa help "*) printf 'help\n' ;;
  *" bwa mem "*) printf '{"records":1}\n' > "${output_root}/result.json" ;;
  *" bwa invalid "*) exit 2 ;;
  *" --version "*) printf 'apptainer version 1.3.0\n' ;;
  *) exit 64 ;;
esac
"#,
    )?;

    let evidence: serde_json::Value = serde_json::from_slice(&fs::read(&execution.evidence_path)?)?;
    assert!(execution.passed, "native execution evidence: {evidence:#}");
    assert_eq!(evidence["status"], "passed");
    assert_eq!(evidence["failure_details"], serde_json::json!([]));
    assert_eq!(evidence["checks"]["negative"]["exit_code"], 2);
    assert_eq!(evidence["metrics"][0]["observed"], 1);
    assert_eq!(evidence["site"]["host"], evidence["runtime"]["host"]);
    assert!(evidence["site"]["host"].as_str().is_some_and(|host| !host.is_empty()));
    assert!(fixture.paths.result_root.join("outputs/result.json").is_file());
    assert!(!fs::read_to_string(fixture.evidence_root.join("scheduler/job.sbatch"))?
        .contains("container_validation.py"));
    Ok(())
}

#[test]
fn native_executor_publishes_failed_evidence_for_malformed_metrics() -> Result<()> {
    let fixture = fixture()?;
    prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let spec = load_container_qualification_slurm_spec(
        &fixture.evidence_root.join("qualification-spec.json"),
    )?;
    let execution = execute_with_fake_runtime(
        &fixture,
        &spec,
        r#"#!/bin/sh
set -eu
output_root=
for argument in "$@"; do
  case "$argument" in
    *:/result:rw) output_root=${argument%:/result:rw} ;;
  esac
done
case " $* " in
  *" bwa version "*) printf '1.0\n' ;;
  *" bwa help "*) printf 'help\n' ;;
  *" bwa mem "*) printf '{}\n' > "${output_root}/result.json" ;;
  *" bwa invalid "*) exit 2 ;;
  *" --version "*) printf 'apptainer version 1.3.0\n' ;;
  *) exit 64 ;;
esac
"#,
    )?;

    let evidence: serde_json::Value = serde_json::from_slice(&fs::read(&execution.evidence_path)?)?;
    assert!(!execution.passed);
    assert_eq!(evidence["status"], "failed");
    assert_eq!(evidence["metrics"][0]["status"], "failed");
    assert!(evidence["metrics"][0]["observed"].is_null());
    assert!(evidence["failures"][0]
        .as_str()
        .is_some_and(|failure| failure.contains("metric records cannot be evaluated")));
    assert_eq!(evidence["failure_details"][0]["class"], "metric_unreadable");
    assert_eq!(evidence["failure_details"][0]["subject"], "records");
    Ok(())
}

fn execute_with_fake_runtime(
    fixture: &Fixture,
    spec: &ContainerQualificationSlurmSpec,
    runtime: &str,
) -> Result<ContainerQualificationExecution> {
    let bin = fixture.root.path().join("bin");
    fs::create_dir(&bin)?;
    write_executable(&bin.join("apptainer"), runtime)?;
    let scratch = fixture.root.path().join("study/scratch/job-123");
    fs::create_dir(&scratch)?;
    let execution_path = fixture.root.path().join("native-execution.json");
    let output = Command::new(std::env::current_exe()?)
        .args(["native_executor_subprocess", "--nocapture"])
        .env("BIJUX_QUALIFICATION_SPEC", fixture.evidence_root.join("qualification-spec.json"))
        .env("BIJUX_QUALIFICATION_EXECUTION", &execution_path)
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.display(),
                std::env::var_os("PATH").unwrap_or_default().to_string_lossy()
            ),
        )
        .env("TMPDIR", &scratch)
        .env("SLURM_JOB_ID", "123")
        .env("SLURM_CPUS_PER_TASK", "2")
        .output()?;
    ensure!(
        output.status.success(),
        "native executor subprocess failed: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let execution: ContainerQualificationExecution =
        serde_json::from_slice(&fs::read(execution_path)?)?;
    ensure!(
        execution.evidence_path == spec.result_root.join("evidence.json"),
        "subprocess returned evidence for a different qualification"
    );
    Ok(execution)
}

#[test]
fn native_executor_subprocess() -> Result<()> {
    let Some(specification) = std::env::var_os("BIJUX_QUALIFICATION_SPEC") else {
        return Ok(());
    };
    let output = PathBuf::from(
        std::env::var_os("BIJUX_QUALIFICATION_EXECUTION")
            .context("native execution output path is absent")?,
    );
    let spec = load_container_qualification_slurm_spec(&PathBuf::from(specification))?;
    let execution = execute_container_qualification(&spec)?;
    fs::write(output, serde_json::to_vec_pretty(&execution)?)?;
    if !execution.passed {
        let evidence: serde_json::Value =
            serde_json::from_slice(&fs::read(&execution.evidence_path)?)?;
        ensure!(evidence["status"] == "failed", "failed execution lacks failed evidence");
    }
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
fn preparation_keeps_qualification_attempts_immutable_and_addressable() -> Result<()> {
    let fixture = fixture()?;
    let first =
        prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let first_spec_path = fixture.evidence_root.join("qualification-spec.json");
    let first_spec = load_container_qualification_slurm_spec(&first_spec_path)?;

    let mut second_request = fixture.request.clone();
    second_request.attempt_id = "attempt-002".to_string();
    let second =
        prepare_container_qualification_slurm_job(&second_request, "2026-08-07T10:00:00Z")?;
    let second_evidence_root = fixture
        .containers_root
        .join("evidence/attempts/run-1200/bwa/attempts/governed-runner-test/attempt-002");
    let second_spec = load_container_qualification_slurm_spec(
        &second_evidence_root.join("qualification-spec.json"),
    )?;

    assert!(first_spec_path.is_file());
    assert_ne!(first, second);
    assert_eq!(first_spec.attempt_id.as_deref(), Some("attempt-001"));
    assert_eq!(second_spec.attempt_id.as_deref(), Some("attempt-002"));
    assert_eq!(first_spec.campaign_id.as_deref(), Some("governed-runner-test"));
    assert_eq!(first_spec.evidence_root, fixture.evidence_root);
    assert_eq!(second_spec.evidence_root, second_evidence_root);
    assert_eq!(first_spec.result_root, fixture.paths.result_root);
    assert_eq!(second_spec.result_root, second_evidence_root);
    assert_eq!(
        prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?,
        first
    );
    Ok(())
}

#[test]
fn historical_v3_specification_remains_loadable_at_its_flat_roots() -> Result<()> {
    let fixture = fixture()?;
    prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let mut spec = load_container_qualification_slurm_spec(
        &fixture.evidence_root.join("qualification-spec.json"),
    )?;
    let evidence_namespace = fixture.containers_root.join("evidence/submissions/run-1200-bwa");
    fs::create_dir_all(&evidence_namespace)?;
    for name in ["config-governance.yml", "config-pipeline.yml", "config-credential.yml"] {
        fs::copy(fixture.evidence_root.join(name), evidence_namespace.join(name))?;
    }
    spec.schema_version = "bijux.runner.container_qualification_slurm_spec.v3".to_string();
    spec.campaign_id = None;
    spec.attempt_id = None;
    spec.governance_path = evidence_namespace.join("config-governance.yml");
    spec.pipeline_path = evidence_namespace.join("config-pipeline.yml");
    spec.credential_reference_path = evidence_namespace.join("config-credential.yml");
    spec.evidence_root = evidence_namespace.clone();
    spec.result_root = fixture.paths.result_namespace.clone();
    let legacy_spec_path = evidence_namespace.join("qualification-spec.json");
    fs::write(&legacy_spec_path, serde_json::to_vec_pretty(&spec)?)?;

    assert_eq!(load_container_qualification_slurm_spec(&legacy_spec_path)?, spec);
    Ok(())
}

#[test]
fn historical_v4_specification_remains_loadable_at_its_attempt_roots() -> Result<()> {
    let fixture = fixture()?;
    prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let mut spec = load_container_qualification_slurm_spec(
        &fixture.evidence_root.join("qualification-spec.json"),
    )?;
    let evidence_root =
        fixture.containers_root.join("evidence/submissions/run-1200-bwa/attempts/attempt-001");
    fs::create_dir_all(&evidence_root)?;
    for name in ["config-governance.yml", "config-pipeline.yml", "config-credential.yml"] {
        fs::copy(fixture.evidence_root.join(name), evidence_root.join(name))?;
    }
    spec.schema_version = "bijux.runner.container_qualification_slurm_spec.v4".to_string();
    spec.campaign_id = None;
    spec.governance_path = evidence_root.join("config-governance.yml");
    spec.pipeline_path = evidence_root.join("config-pipeline.yml");
    spec.credential_reference_path = evidence_root.join("config-credential.yml");
    spec.evidence_root = evidence_root.clone();
    spec.result_root = fixture.paths.result_namespace.join("attempts/attempt-001");
    let legacy_spec_path = evidence_root.join("qualification-spec.json");
    fs::write(&legacy_spec_path, serde_json::to_vec_pretty(&spec)?)?;

    assert_eq!(load_container_qualification_slurm_spec(&legacy_spec_path)?, spec);
    Ok(())
}

#[test]
fn legacy_python_scheduler_specifications_are_rejected() -> Result<()> {
    let fixture = fixture()?;
    prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let current = fixture.evidence_root.join("qualification-spec.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&current)?)?;

    value["schema_version"] =
        serde_json::Value::String("bijux.runner.container_qualification_slurm_spec.v2".to_string());
    value["attempt_id"] = serde_json::Value::Null;
    let legacy_schema = fixture.root.path().join("legacy-schema.json");
    fs::write(&legacy_schema, serde_json::to_vec_pretty(&value)?)?;
    assert!(load_container_qualification_slurm_spec(&legacy_schema).is_err());

    value["schema_version"] =
        serde_json::Value::String("bijux.runner.container_qualification_slurm_spec.v5".to_string());
    value["attempt_id"] = serde_json::Value::String("attempt-001".to_string());
    value["executor_script_path"] = serde_json::Value::String(
        fixture.frozen_repository.join("legacy-worker.py").display().to_string(),
    );
    value["executor_script_sha256"] = serde_json::Value::String("a".repeat(64));
    let legacy_worker = fixture.root.path().join("legacy-worker.json");
    fs::write(&legacy_worker, serde_json::to_vec_pretty(&value)?)?;
    assert!(load_container_qualification_slurm_spec(&legacy_worker).is_err());
    Ok(())
}

#[test]
fn preparation_rejects_malformed_qualification_attempt_identity() -> Result<()> {
    let fixture = fixture()?;
    for attempt_id in ["attempt-000", "attempt-01", "attempt-one", "../attempt-002"] {
        let mut request = fixture.request.clone();
        request.attempt_id = attempt_id.to_string();
        let Err(error) =
            prepare_container_qualification_slurm_job(&request, "2026-08-07T09:50:00Z")
        else {
            anyhow::bail!("malformed attempt identity was accepted: {attempt_id}");
        };
        assert!(
            error.to_string().contains("qualification attempt identity"),
            "unexpected refusal for {attempt_id}: {error:#}"
        );
    }
    Ok(())
}

#[test]
fn preparation_canonicalizes_governed_root_parent_aliases() -> Result<()> {
    let fixture = fixture()?;
    let containers = &fixture.containers_root;
    let study = containers.parent().context("resolve fixture study root")?;
    let study_alias = fixture.root.path().join("study-alias");
    symlink(study, &study_alias)?;

    let mut governance: ContainerGovernance =
        serde_json::from_slice(&fs::read(&fixture.request.governance_path)?)?;
    governance.paths.study_root = study_alias.clone();
    governance.paths.containers_root = study_alias.join("containers");
    governance.paths.data_root = study_alias.join("data");
    governance.paths.results_root = study_alias.join("results");
    governance.paths.finalization_root = study_alias.join("finalization");
    fs::write(&fixture.request.governance_path, serde_json::to_vec_pretty(&governance)?)?;

    prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")?;
    let spec = load_container_qualification_slurm_spec(
        &fixture.evidence_root.join("qualification-spec.json"),
    )?;
    assert_eq!(spec.evidence_root, fixture.evidence_root);
    Ok(())
}

#[test]
fn preparation_rejects_non_native_executor_authority() -> Result<()> {
    let fixture = fixture()?;
    let executor = fixture.root.path().join("qualification-driver");
    write_executable(&executor, "#!/bin/sh\nexit 0\n")?;
    let mut request = fixture.request.clone();
    request.executor_program_path = executor;

    let Err(error) = prepare_container_qualification_slurm_job(&request, "2026-08-07T09:50:00Z")
    else {
        anyhow::bail!("non-native qualification executor was accepted");
    };
    assert!(
        error.to_string().contains("must be the pinned bijux-dna-dev program"),
        "unexpected executor refusal: {error:#}"
    );
    Ok(())
}

#[test]
fn preparation_rejects_symlinked_governed_root() -> Result<()> {
    let fixture = fixture()?;
    let containers = &fixture.containers_root;
    let containers_alias =
        containers.parent().context("resolve fixture study root")?.join("containers-alias");
    symlink(containers, &containers_alias)?;

    let mut governance: ContainerGovernance =
        serde_json::from_slice(&fs::read(&fixture.request.governance_path)?)?;
    governance.paths.containers_root = containers_alias;
    fs::write(&fixture.request.governance_path, serde_json::to_vec_pretty(&governance)?)?;

    let Err(error) =
        prepare_container_qualification_slurm_job(&fixture.request, "2026-08-07T09:50:00Z")
    else {
        anyhow::bail!("symlinked governed root was accepted");
    };
    assert!(
        error.to_string().contains("not a regular directory"),
        "unexpected symlink refusal: {error:#}"
    );
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
        attempt_id: "attempt-001".to_owned(),
        hostname: "test-node".to_owned(),
        governance_path: fixture.request.governance_path.clone(),
        source_pipeline_path: source_pipeline.canonicalize()?,
        replay_pipeline_path: replay_pipeline.canonicalize()?,
        credential_reference_path: fixture.request.credential_reference_path.clone(),
        executor_program_path: fixture.request.executor_program_path.clone(),
    };
    let prepared =
        prepare_historical_container_qualification_replay(&replay_request, "2026-08-07T09:50:00Z")?;
    let replay_evidence_root = fixture
        .containers_root
        .join("evidence/attempts/run-1200/bwa/attempts/governed-runner-replay-test/attempt-001");
    let spec = load_container_qualification_slurm_spec(
        &replay_evidence_root.join("qualification-spec.json"),
    )?;
    assert_eq!(spec.submission_kind, ContainerQualificationSubmissionKind::HistoricalReplay);
    assert_eq!(spec.schema_version, "bijux.runner.container_qualification_slurm_spec.v5");
    assert_eq!(spec.campaign_id.as_deref(), Some("governed-runner-replay-test"));
    assert_eq!(spec.attempt_id.as_deref(), Some("attempt-001"));
    assert_eq!(spec.result_root, replay_evidence_root);
    assert!(spec.result_root.join("outputs").is_dir());
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

    let admission = admit_container_qualification_collection(&spec, "2026-08-07T09:52:30Z")?;
    assert_eq!(admission.run_id, spec.run_id);
    assert_eq!(admission.tool_id, spec.tool_id);
    assert_eq!(admission.campaign_id, "governed-runner-test");
    assert_eq!(admission.attempt_id, "attempt-001");
    assert_eq!(admit_container_qualification_collection(&spec, "2026-08-07T09:52:45Z")?, admission);
    assert_eq!(load_container_qualification_admission(&spec)?, Some(admission));

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
    assert!(load_container_qualification_admission(&spec)
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
    assert!(admit_container_qualification_collection(&spec, "2026-08-07T09:53:00Z")
        .is_err_and(|error| error.to_string().contains("terminal passing")));
    write_executable(&scheduler.squeue, "#!/bin/sh\nexit 99\n")?;
    assert!(collect_container_qualification_slurm_job(&scheduler, &spec, "2026-08-07T09:53:00Z")?
        .evidence
        .is_some());
    Ok(())
}

#[test]
fn failed_scheduler_attempt_is_archived_without_execution_evidence() -> Result<()> {
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
        "#!/bin/sh\nprintf '%s\n' '456789|FAILED|1:0|12|00:00:10|128K|n001|None'\n",
    )?;
    fs::write(&prepared.request.stdout_path, b"")?;
    fs::write(&prepared.request.stderr_path, b"executor did not start\n")?;
    collect_container_qualification_slurm_job(&scheduler, &spec, "2026-08-07T09:52:00Z")?;
    assert!(!spec.result_root.join("evidence.json").exists());

    let archived = archive_failed_container_qualification_attempt(&spec, "2026-08-07T09:54:00Z")?;

    assert!(!spec.evidence_root.exists());
    assert!(!spec.result_root.exists());
    assert!(archived.archive_receipt_path.is_file());
    let persisted: bijux_dna_infra::AttemptArchiveReceiptV1 =
        serde_json::from_slice(&fs::read(&archived.archive_receipt_path)?)?;
    assert_eq!(persisted.owner, bijux_dna_infra::AttemptEvidenceOwner::Containers);
    assert_eq!(persisted.identity.run_directory, "run-1200");
    assert_eq!(persisted.identity.tool_id, "bwa");
    assert_eq!(persisted.identity.campaign_id, "governed-runner-test");
    assert_eq!(persisted.identity.attempt_id, "attempt-001");
    assert_eq!(persisted.disposition, bijux_dna_infra::AttemptArchiveDisposition::Failed);
    assert!(persisted.files.iter().any(|file| file.path == "scheduler-evidence.json"));
    assert!(persisted.files.iter().any(|file| file.path == "slurm-accounting.json"));
    for file in &persisted.files {
        let path = archived.archive_dir.join(&file.path);
        assert_eq!(fs::metadata(&path)?.len(), file.size_bytes);
        assert_eq!(bijux_dna_infra::hash_file_sha256(&path)?, file.sha256);
    }

    let restored = rollback_failed_container_qualification_attempt_archive(
        &fixture.containers_root.join("evidence/attempts"),
        &spec,
        "2026-08-07T09:54:00Z",
        "2026-08-07T09:55:00Z",
    )?;
    assert!(restored.source.attempt_dir.is_dir());
    assert!(restored.rollback_receipt_path.is_file());
    assert!(spec.result_root.join("slurm-accounting.json").is_file());
    Ok(())
}

#[test]
fn failed_attempt_archival_rejects_symlinks_before_moving_roots() -> Result<()> {
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
        "#!/bin/sh\nprintf '%s\n' '456789|FAILED|1:0|12|00:00:10|128K|n001|None'\n",
    )?;
    fs::write(&prepared.request.stdout_path, b"")?;
    fs::write(&prepared.request.stderr_path, b"executor did not start\n")?;
    collect_container_qualification_slurm_job(&scheduler, &spec, "2026-08-07T09:52:00Z")?;
    symlink(
        spec.result_root.join("slurm-accounting.json"),
        spec.result_root.join("accounting-link.json"),
    )?;

    let Err(error) = archive_failed_container_qualification_attempt(&spec, "2026-08-07T09:54:00Z")
    else {
        anyhow::bail!("symlink-contaminated evidence was archived");
    };
    assert!(format!("{error:#}").contains("symlink"));
    assert!(spec.evidence_root.is_dir());
    assert!(spec.result_root.is_dir());
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
    result_namespace: PathBuf,
    result_root: PathBuf,
}

struct Fixture {
    root: tempfile::TempDir,
    containers_root: PathBuf,
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

    let paths = fixture_paths(&containers, &data, &results)?;
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
        fixture_hostname_regex()?,
    );
    let governance_path = root.path().join("governance.yml");
    fs::write(&governance_path, serde_json::to_vec_pretty(&governance)?)?;
    let pipeline_path = root.path().join("pipeline.yml");
    let fixture_stub = Fixture {
        root,
        containers_root: containers.clone(),
        active_repository,
        frozen_repository,
        commit,
        request: ContainerQualificationPreparationRequest {
            attempt_id: String::new(),
            hostname: String::new(),
            governance_path: PathBuf::new(),
            pipeline_path: PathBuf::new(),
            credential_reference_path: PathBuf::new(),
            executor_program_path: PathBuf::new(),
        },
        evidence_root: containers
            .join("evidence/attempts/run-1200/bwa/attempts/governed-runner-test/attempt-001"),
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
    let executor_program_path = fixture_stub.root.path().join("bijux-dna-dev");
    write_executable(&executor_program_path, "#!/bin/sh\nexit 0\n")?;
    Ok(Fixture {
        request: ContainerQualificationPreparationRequest {
            attempt_id: "attempt-001".to_string(),
            hostname: "test-node".to_string(),
            governance_path,
            pipeline_path,
            credential_reference_path,
            executor_program_path,
        },
        ..fixture_stub
    })
}

fn fixture_paths(containers: &Path, data: &Path, results: &Path) -> Result<FixturePaths> {
    let image = containers.join("apptainer/bwa.sif");
    fs::write(&image, b"immutable image")?;
    let input = data.join("reads.fastq.gz");
    fs::write(&input, b"immutable reads")?;
    let upstream = results.join("upstream/result.json");
    fs::create_dir_all(upstream.parent().context("upstream parent")?)?;
    fs::write(&upstream, b"immutable upstream result")?;
    let result_namespace = results.join("container-validation/run-1200-bwa");
    Ok(FixturePaths {
        image_sha256: bijux_dna_infra::hash_file_sha256(&image)?,
        image,
        input_sha256: bijux_dna_infra::hash_file_sha256(&input)?,
        input,
        upstream_sha256: bijux_dna_infra::hash_file_sha256(&upstream)?,
        upstream,
        result_root: containers
            .join("evidence/attempts/run-1200/bwa/attempts/governed-runner-test/attempt-001"),
        result_namespace,
    })
}

fn fixture_hostname_regex() -> Result<String> {
    let output = Command::new("hostname").output()?;
    ensure!(output.status.success(), "resolve fixture hostname");
    let hostname = String::from_utf8(output.stdout)?.trim().to_string();
    ensure!(!hostname.is_empty(), "fixture hostname is empty");
    Ok(format!("(?:test-node|{})", regex::escape(&hostname)))
}

fn setup_repositories(active_repository: &Path, frozen_root: &Path) -> Result<(String, PathBuf)> {
    git(active_repository, &["init", "--quiet"])?;
    fs::write(active_repository.join("README.md"), b"governed repository\n")?;
    git(active_repository, &["add", "README.md"])?;
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

fn governance(
    paths: ContainerPathPolicy,
    study: &Path,
    hostname_regex: String,
) -> ContainerGovernance {
    ContainerGovernance {
        schema_version: "bijux.container.governance.v1".to_string(),
        site: ContainerSitePolicy { id: "test-site".to_string(), hostname_regex },
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
                "result_root": fixture.paths.result_namespace,
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
        .result_namespace
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
    write_executable(&paths.sbatch, &format!("#!/bin/sh\n{sbatch_body}"))?;
    Ok(paths)
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
