use std::path::Path;
use std::process::Command;

use bijux_dna_containers::api::{
    freeze_repository, ContainerGovernance, ContainerPathPolicy, ContainerPreflightPolicy,
    ContainerRuntimePolicy, ContainerSitePolicy, ContainerSlurmPolicy, RepositoryFreezeRequest,
    RepositoryFreezeScope, CONTAINER_GOVERNANCE_SCHEMA_V1,
};
use bijux_dna_runner::api::{
    collect_image_campaign_slurm_job, observe_image_campaign_slurm_job,
    plan_image_campaign_slurm_spec, prepare_image_campaign_slurm_job,
    submit_prepared_image_campaign_job, ImageCampaignEvidenceStatus, ImageCampaignSlurmPlanRequest,
    ImageCampaignSlurmResources, ImageCampaignSlurmSpec, SlurmClientPaths, SlurmJobState,
    IMAGE_CAMPAIGN_SLURM_SPEC_SCHEMA_V1,
};

#[cfg(unix)]
use crate::scheduler_executable::write_executable;

struct Fixture {
    root: tempfile::TempDir,
    spec: ImageCampaignSlurmSpec,
}

#[cfg(unix)]
fn fixture() -> anyhow::Result<Fixture> {
    let root = tempfile::tempdir()?;
    let canonical_root = root.path().canonicalize()?;
    let repository = canonical_root.join("repository");
    let study_root = canonical_root.join("study");
    std::fs::create_dir_all(repository.join("containers/contracts"))?;
    std::fs::create_dir(&study_root)?;
    std::fs::copy(
        crate::support::repo_root()?.join("containers/contracts/catalog.toml"),
        repository.join("containers/contracts/catalog.toml"),
    )?;
    let executable = canonical_root.join("bijux-dna-dev");
    let apptainer = canonical_root.join("apptainer");
    write_executable(&executable, "#!/bin/sh\nexit 0\n")?;
    write_executable(&apptainer, "#!/bin/sh\nexit 0\n")?;
    run_git(&repository, &["init", "-q"])?;
    run_git(&repository, &["add", "containers/contracts/catalog.toml"])?;
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
    let repository_commit = git_output(&repository, &["rev-parse", "HEAD"])?;
    let attempt_id = "attempt-001".to_string();
    let attempt_root = study_root
        .join("containers/evidence/image-campaign-submissions")
        .join(&repository_commit)
        .join("attempts")
        .join(&attempt_id);
    let spec = ImageCampaignSlurmSpec {
        schema_version: IMAGE_CAMPAIGN_SLURM_SPEC_SCHEMA_V1.to_string(),
        run_id: "run-1000".to_string(),
        attempt_id,
        repository_path: repository,
        repository_commit,
        executable_sha256: bijux_dna_infra::hash_file_sha256(&executable)?,
        executable_path: executable.canonicalize()?,
        study_root: study_root.canonicalize()?,
        apptainer_sha256: bijux_dna_infra::hash_file_sha256(&apptainer)?,
        apptainer_path: apptainer.canonicalize()?,
        attempt_root,
        selected_tools: vec!["bijux_dna".to_string(), "picard".to_string()],
        worker_count: 2,
        resources: ImageCampaignSlurmResources {
            account: "lu2024-12-24".to_string(),
            partition: "compute".to_string(),
            cpus: 2,
            memory_mb: 16_384,
            walltime: "02:00:00".to_string(),
        },
    };
    Ok(Fixture { root, spec })
}

#[cfg(unix)]
#[test]
fn image_campaign_planning_derives_source_and_catalog_authorities() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let request = ImageCampaignSlurmPlanRequest {
        repository_path: fixture.spec.repository_path.clone(),
        executable_path: fixture.spec.executable_path.clone(),
        study_root: fixture.spec.study_root.clone(),
        apptainer_path: fixture.spec.apptainer_path.clone(),
        attempt_id: fixture.spec.attempt_id.clone(),
        selected_tools: vec!["picard".to_string(), "bijux_dna".to_string()],
        worker_count: fixture.spec.worker_count,
        resources: fixture.spec.resources.clone(),
    };

    let planned = plan_image_campaign_slurm_spec(&request)?;
    assert_eq!(planned, fixture.spec);
    assert_eq!(plan_image_campaign_slurm_spec(&request)?, planned);
    assert_eq!(
        bijux_dna_runner::api::load_image_campaign_slurm_spec(
            &planned.attempt_root.join("image-campaign-spec.json"),
        )?,
        planned
    );

    let mut invalid = request;
    invalid.attempt_id = "attempt-unknown".to_string();
    invalid.selected_tools = vec!["not-a-retained-tool".to_string()];
    assert!(plan_image_campaign_slurm_spec(&invalid)
        .is_err_and(|error| error.to_string().contains("unknown or non-retained")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn image_campaign_material_is_runner_owned_and_restart_safe() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let first = prepare_image_campaign_slurm_job(&fixture.spec, "2026-08-09T07:30:00Z")?;
    let recovered = prepare_image_campaign_slurm_job(&fixture.spec, "2026-08-09T07:30:00Z")?;
    assert_eq!(first, recovered);
    let script = std::fs::read_to_string(&first.request.script_path)?;
    assert!(script.contains("'containers' 'run' 'image-campaign' '--' 'execute'"));
    assert!(script.contains("'bijux_dna,picard'"));
    assert!(script.contains("$(date -u +%Y-%m-%dT%H:%M:%SZ)"));
    assert!(!script.contains("image_campaign_job_completion"));
    assert!(!script.contains("completion.json.partial"));
    assert!(Command::new("bash").arg("-n").arg(&first.request.script_path).status()?.success());
    Ok(())
}

#[cfg(unix)]
#[test]
fn image_campaign_preparation_rejects_intermediate_study_symlinks() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let containers = fixture.spec.study_root.join("containers");
    let external = fixture.root.path().join("external-evidence");
    std::fs::create_dir(&containers)?;
    std::fs::create_dir(&external)?;
    std::os::unix::fs::symlink(&external, containers.join("evidence"))?;
    let error = prepare_image_campaign_slurm_job(&fixture.spec, "2026-08-09T07:30:00Z")
        .err()
        .ok_or_else(|| anyhow::anyhow!("intermediate STUDY symlink was accepted"))?;
    assert!(format!("{error:#}").contains("non-symlink directory"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn image_campaign_preparation_rejects_executable_drift() -> anyhow::Result<()> {
    let fixture = fixture()?;
    write_executable(&fixture.spec.executable_path, "#!/bin/sh\nexit 1\n")?;
    assert!(prepare_image_campaign_slurm_job(&fixture.spec, "2026-08-09T07:30:00Z")
        .is_err_and(|error| error.to_string().contains("digest differs")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn submission_is_duplicate_safe_and_observation_is_non_waiting() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let prepared = prepare_image_campaign_slurm_job(&fixture.spec, "2026-08-09T07:30:00Z")?;
    let calls = fixture.root.path().join("sbatch.calls");
    let scheduler = scheduler(
        fixture.root.path(),
        &format!("printf x >> {}\nprintf '456789\\n'\n", shell_literal(&calls)),
    )?;
    write_executable(&scheduler.squeue, "#!/bin/sh\nprintf 'RUNNING\\n'\n")?;
    let first = submit_prepared_image_campaign_job(&scheduler, &prepared)?;
    let recovered = submit_prepared_image_campaign_job(&scheduler, &prepared)?;
    let observation =
        observe_image_campaign_slurm_job(&scheduler, &fixture.spec, "2026-08-09T07:31:00Z")?;
    assert_eq!(first, recovered);
    assert_eq!(std::fs::read_to_string(calls)?, "x");
    assert_eq!(observation.state, SlurmJobState::Running);
    write_executable(&fixture.spec.executable_path, "#!/bin/sh\nexit 1\n")?;
    let after_executable_advance =
        observe_image_campaign_slurm_job(&scheduler, &fixture.spec, "2026-08-09T07:31:20Z")?;
    assert_eq!(after_executable_advance.state, SlurmJobState::Running);
    let mut conflicting = fixture.spec.clone();
    conflicting.resources.memory_mb += 1;
    assert!(observe_image_campaign_slurm_job(&scheduler, &conflicting, "2026-08-09T07:31:30Z")
        .is_err_and(|error| error.to_string().contains("immutable attempt evidence")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn submission_rejects_a_source_advance_after_preparation() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let prepared = prepare_image_campaign_slurm_job(&fixture.spec, "2026-08-09T07:30:00Z")?;
    advance_source(&fixture.spec.repository_path)?;
    let scheduler = scheduler(fixture.root.path(), "printf '456789\\n'\n")?;
    assert!(submit_prepared_image_campaign_job(&scheduler, &prepared)
        .is_err_and(|error| error.to_string().contains("repository commit differs")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn terminal_scheduler_failure_survives_source_advance() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let prepared = prepare_image_campaign_slurm_job(&fixture.spec, "2026-08-09T07:30:00Z")?;
    let scheduler = scheduler(fixture.root.path(), "printf '456789\\n'\n")?;
    let handle = submit_prepared_image_campaign_job(&scheduler, &prepared)?;
    advance_source(&fixture.spec.repository_path)?;
    std::fs::write(&handle.stdout_path, b"campaign stdout\n")?;
    std::fs::write(&handle.stderr_path, b"campaign failed\n")?;
    write_executable(&scheduler.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &scheduler.sacct,
        "#!/bin/sh\nprintf '%s\\n' '456789|FAILED|42:0|8|00:00:06|32M|cn001|build failure'\n",
    )?;
    let first =
        collect_image_campaign_slurm_job(&scheduler, &fixture.spec, "2026-08-09T07:32:00Z")?;
    let recovered =
        collect_image_campaign_slurm_job(&scheduler, &fixture.spec, "2026-08-09T07:42:00Z")?;
    assert_ne!(first.observation, recovered.observation);
    assert_eq!(first.evidence, recovered.evidence);
    let evidence = first.evidence.ok_or_else(|| anyhow::anyhow!("missing evidence"))?;
    assert_eq!(evidence.status, ImageCampaignEvidenceStatus::Failed);
    assert_eq!(evidence.accounting.exit_code, 42);
    assert!(evidence.failure.as_deref().is_some_and(|value| value.contains("scheduler state")));
    assert!(fixture.spec.attempt_root.join("scheduler/collection.json").is_file());
    Ok(())
}

#[cfg(unix)]
#[test]
fn completed_scheduler_attempt_uses_frozen_source_after_advance() -> anyhow::Result<()> {
    let fixture = fixture()?;
    let prepared = prepare_image_campaign_slurm_job(&fixture.spec, "2026-08-09T07:30:00Z")?;
    let scheduler = scheduler(fixture.root.path(), "printf '456789\\n'\n")?;
    let handle = submit_prepared_image_campaign_job(&scheduler, &prepared)?;
    advance_source(&fixture.spec.repository_path)?;
    freeze_source_snapshot(&fixture.spec)?;
    std::fs::write(&handle.stdout_path, b"campaign stdout\n")?;
    std::fs::write(&handle.stderr_path, b"")?;
    write_executable(&scheduler.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &scheduler.sacct,
        "#!/bin/sh\nprintf '%s\\n' '456789|COMPLETED|0:0|8|00:00:06|32M|cn001|'\n",
    )?;

    let collection =
        collect_image_campaign_slurm_job(&scheduler, &fixture.spec, "2026-08-09T07:32:00Z")?;
    let evidence = collection.evidence.ok_or_else(|| anyhow::anyhow!("missing evidence"))?;
    assert_eq!(evidence.status, ImageCampaignEvidenceStatus::Failed);
    assert!(evidence
        .failure
        .as_deref()
        .is_some_and(|value| value.contains("campaign receipt admission failed")));
    Ok(())
}

fn advance_source(repository: &Path) -> anyhow::Result<()> {
    std::fs::write(repository.join("advanced.txt"), b"advanced\n")?;
    run_git(repository, &["add", "advanced.txt"])?;
    run_git(
        repository,
        &[
            "-c",
            "user.name=Bijux Test",
            "-c",
            "user.email=bijux@example.invalid",
            "commit",
            "-qm",
            "advance source",
        ],
    )
}

fn freeze_source_snapshot(spec: &ImageCampaignSlurmSpec) -> anyhow::Result<()> {
    freeze_repository(&RepositoryFreezeRequest {
        governance: ContainerGovernance {
            schema_version: CONTAINER_GOVERNANCE_SCHEMA_V1.to_owned(),
            site: ContainerSitePolicy {
                id: "test-site".to_owned(),
                hostname_regex: "^test-host$".to_owned(),
            },
            paths: ContainerPathPolicy {
                study_root: spec.study_root.clone(),
                containers_root: spec.study_root.join("containers"),
                data_root: spec.study_root.join("data"),
                results_root: spec.study_root.join("results"),
                finalization_root: spec.study_root.join("evidence/finalization"),
                active_repository: spec.repository_path.clone(),
                frozen_repositories: spec.study_root.join("data/evidence/source-checkouts"),
            },
            runtime: ContainerRuntimePolicy { apptainer_architecture: "linux/amd64".to_owned() },
            slurm: ContainerSlurmPolicy {
                account: "science".to_owned(),
                scratch_environment: Some("TEST_SCRATCH".to_owned()),
                scratch_root: None,
                partitions: vec!["compute".to_owned()],
                quota_command: vec!["true".to_owned()],
            },
            preflight: ContainerPreflightPolicy {
                minimum_available_bytes: 1,
                minimum_available_inodes: 1,
                maximum_clock_skew_seconds: 30,
                required_commands: vec!["git".to_owned()],
                required_contracts: vec!["containers/contracts/catalog.toml".to_owned()],
            },
        },
        hostname: "test-host".to_owned(),
        commit: spec.repository_commit.clone(),
        scope: RepositoryFreezeScope::Complete,
        checkout_paths: Vec::new(),
        source_namespace: "source-freezes".to_owned(),
    })?;
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
