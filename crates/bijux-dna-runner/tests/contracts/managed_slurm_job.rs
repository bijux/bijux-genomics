use std::path::Path;
use std::process::Command;

use bijux_dna_runner::api::{
    collect_managed_slurm_job, load_managed_slurm_job_spec, observe_managed_slurm_job,
    prepare_managed_slurm_job, submit_prepared_managed_slurm_job, ManagedSlurmJobResources,
    ManagedSlurmJobSpec, SlurmClientPaths, SlurmDependencyCondition, SlurmJobState,
    MANAGED_SLURM_JOB_SPEC_SCHEMA_V1,
};

#[cfg(unix)]
use crate::scheduler_executable::write_executable;

struct Fixture {
    root: tempfile::TempDir,
    spec: ManagedSlurmJobSpec,
}

#[cfg(unix)]
fn fixture() -> anyhow::Result<Fixture> {
    let root = tempfile::tempdir()?;
    let root_path = root.path().canonicalize()?;
    let repository = root_path.join("repository");
    std::fs::create_dir(&repository)?;
    std::fs::write(repository.join("authority.txt"), b"authority\n")?;
    run_git(&repository, &["init", "-q"])?;
    run_git(&repository, &["add", "authority.txt"])?;
    run_git(
        &repository,
        &[
            "-c",
            "user.name=Bijux Test",
            "-c",
            "user.email=bijux@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    )?;
    let repository = repository.canonicalize()?;
    let worker = root_path.join("inventory-worker");
    write_executable(&worker, "#!/bin/sh\nprintf 'complete\\n'\n")?;
    let attempt_root = root_path.join("attempt");
    std::fs::create_dir(&attempt_root)?;
    let spec = ManagedSlurmJobSpec {
        schema_version: MANAGED_SLURM_JOB_SPEC_SCHEMA_V1.to_string(),
        run_id: "container-inventory".to_string(),
        attempt_id: "attempt-001".to_string(),
        repository_commit: git_output(&repository, &["rev-parse", "HEAD"])?,
        repository_path: repository,
        worker_sha256: bijux_dna_infra::hash_file_sha256(&worker)?,
        worker_path: worker.canonicalize()?,
        attempt_root: attempt_root.canonicalize()?,
        dependencies: Vec::new(),
        dependency_condition: SlurmDependencyCondition::AfterOk,
        resources: ManagedSlurmJobResources {
            account: "lu2024-12-24".to_string(),
            partition: "compute".to_string(),
            cpus: 2,
            memory_mb: 4096,
            walltime: "00:30:00".to_string(),
        },
        submitted_at_utc: "2026-08-11T10:00:00Z".to_string(),
    };
    Ok(Fixture { root, spec })
}

#[cfg(unix)]
#[test]
fn preparation_binds_clean_source_worker_and_non_waiting_script() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let first = prepare_managed_slurm_job(&fixture.spec)?;
    let recovered = prepare_managed_slurm_job(&fixture.spec)?;
    assert_eq!(first, recovered);
    let script = std::fs::read_to_string(&first.request.script_path)?;
    assert!(script.contains("exec \"${WORKER}\""));
    assert!(script.contains(&fixture.spec.repository_commit));
    assert!(script.contains(&fixture.spec.worker_sha256));
    assert!(!script.contains("sleep "));
    assert!(!script.contains("while "));
    assert!(!script.contains("until "));
    assert!(Command::new("bash").arg("-n").arg(&first.request.script_path).status()?.success());
    let loaded = load_managed_slurm_job_spec(
        &fixture.spec.attempt_root.join("managed-slurm-job-spec.json"),
    )?;
    assert_eq!(loaded, fixture.spec);
    Ok(())
}

#[cfg(unix)]
#[test]
fn preparation_rejects_worker_drift_and_repository_owned_attempts() -> anyhow::Result<()> {
    let worker_drift = fixture()?;
    std::fs::write(&worker_drift.spec.worker_path, b"#!/bin/sh\nprintf 'changed\\n'\n")?;
    let Err(error) = prepare_managed_slurm_job(&worker_drift.spec) else {
        anyhow::bail!("worker drift was accepted");
    };
    assert!(error.to_string().contains("worker identity differs"));

    let mut repository_attempt = fixture()?;
    repository_attempt.spec.attempt_root = repository_attempt.spec.repository_path.clone();
    let Err(error) = prepare_managed_slurm_job(&repository_attempt.spec) else {
        anyhow::bail!("repository-owned attempt root was accepted");
    };
    assert!(error.to_string().contains("attempt root must be outside"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn submission_is_duplicate_safe_and_observation_is_one_shot() -> anyhow::Result<()> {
    let fixture = fixture()?;
    prepare_managed_slurm_job(&fixture.spec)?;
    let calls = fixture.root.path().join("sbatch.calls");
    let scheduler = scheduler(
        fixture.root.path(),
        &format!("printf x >> {}\nprintf '456789\\n'\n", shell_literal(&calls)),
    )?;
    write_executable(&scheduler.squeue, "#!/bin/sh\nprintf 'RUNNING\\n'\n")?;
    let first = submit_prepared_managed_slurm_job(&scheduler, &fixture.spec)?;
    let recovered = submit_prepared_managed_slurm_job(&scheduler, &fixture.spec)?;
    let observation = observe_managed_slurm_job(&scheduler, &fixture.spec, "2026-08-11T10:01:00Z")?;
    assert_eq!(first, recovered);
    assert_eq!(std::fs::read_to_string(calls)?, "x");
    assert_eq!(observation.state, SlurmJobState::Running);
    Ok(())
}

#[cfg(unix)]
#[test]
fn terminal_collection_is_immutable_across_later_observations() -> anyhow::Result<()> {
    let fixture = fixture()?;
    prepare_managed_slurm_job(&fixture.spec)?;
    let scheduler = scheduler(fixture.root.path(), "printf '456789\\n'\n")?;
    let handle = submit_prepared_managed_slurm_job(&scheduler, &fixture.spec)?;
    std::fs::write(&handle.stdout_path, b"worker stdout\n")?;
    std::fs::write(&handle.stderr_path, b"")?;
    write_executable(&scheduler.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &scheduler.sacct,
        "#!/bin/sh\nprintf '%s\\n' '456789|COMPLETED|0:0|12|00:00:10|128K|n001|None'\n",
    )?;
    let first = collect_managed_slurm_job(&scheduler, &fixture.spec, "2026-08-11T10:02:00Z")?;
    let recovered = collect_managed_slurm_job(&scheduler, &fixture.spec, "2026-08-11T10:12:00Z")?;
    assert_eq!(first.evidence, recovered.evidence);
    assert!(first.evidence.is_some());
    assert!(fixture.spec.attempt_root.join("scheduler/collection.json").is_file());
    Ok(())
}

#[cfg(unix)]
fn scheduler(root: &Path, sbatch_body: &str) -> anyhow::Result<SlurmClientPaths> {
    let paths = SlurmClientPaths {
        sbatch: root.join("sbatch"),
        squeue: root.join("squeue"),
        sacct: root.join("sacct"),
        scancel: root.join("scancel"),
    };
    write_executable(&paths.sbatch, &format!("#!/bin/sh\n{sbatch_body}"))?;
    Ok(paths)
}

fn run_git(repository: &Path, arguments: &[&str]) -> anyhow::Result<()> {
    let status = Command::new("git").arg("-C").arg(repository).args(arguments).status()?;
    anyhow::ensure!(status.success(), "git command failed");
    Ok(())
}

fn git_output(repository: &Path, arguments: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git").arg("-C").arg(repository).args(arguments).output()?;
    anyhow::ensure!(output.status.success(), "git command failed");
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn shell_literal(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}
