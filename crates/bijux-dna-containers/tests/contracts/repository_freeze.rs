use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};
use bijux_dna_containers::api::{
    freeze_repository, load_container_governance, validate_container_governance,
    ContainerGovernance, ContainerPathPolicy, ContainerPreflightPolicy, ContainerRuntimePolicy,
    ContainerSitePolicy, ContainerSlurmPolicy, RepositoryFreezeRequest, RepositoryFreezeScope,
    CONTAINER_GOVERNANCE_SCHEMA_V1,
};

fn git(repository: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .output()
        .with_context(|| format!("run git {}", arguments.join(" ")))?;
    ensure!(
        output.status.success(),
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn fixture(root: &Path) -> Result<(ContainerGovernance, String)> {
    let active = root.join("repository");
    let study = root.join("study");
    std::fs::create_dir_all(&active)?;
    std::fs::create_dir_all(study.join("containers"))?;
    std::fs::create_dir_all(study.join("data"))?;
    std::fs::create_dir_all(study.join("results"))?;
    std::fs::create_dir_all(study.join("evidence/finalization"))?;
    git(&active, &["init", "--quiet"])?;
    git(&active, &["config", "user.name", "Bijux Test"])?;
    git(&active, &["config", "user.email", "test@bijux.invalid"])?;
    for (path, content) in [
        ("benchmarks/contracts/schema.json", "{}\n"),
        ("benchmarks/execution/adapter.sh", "#!/bin/sh\n"),
        ("containers/contracts/governance.yml", "schema_version: test\n"),
        ("other/source.txt", "complete tree\n"),
    ] {
        let destination = active.join(path);
        std::fs::create_dir_all(destination.parent().context("fixture path has no parent")?)?;
        std::fs::write(destination, content)?;
    }
    git(&active, &["add", "."])?;
    git(&active, &["commit", "--quiet", "-m", "fixture"])?;
    let commit = git(&active, &["rev-parse", "HEAD"])?;
    let governance = ContainerGovernance {
        schema_version: CONTAINER_GOVERNANCE_SCHEMA_V1.to_string(),
        site: ContainerSitePolicy {
            id: "test-cluster".to_string(),
            hostname_regex: r"^worker-[0-9]+$".to_string(),
        },
        paths: ContainerPathPolicy {
            study_root: study.clone(),
            containers_root: study.join("containers"),
            data_root: study.join("data"),
            results_root: study.join("results"),
            finalization_root: study.join("evidence/finalization"),
            active_repository: active,
            frozen_repositories: root.join("frozen-repositories"),
        },
        runtime: ContainerRuntimePolicy { apptainer_architecture: "linux/amd64".to_string() },
        slurm: ContainerSlurmPolicy {
            account: "science".to_string(),
            scratch_environment: Some("SNIC_TMP".to_string()),
            scratch_root: None,
            partitions: vec!["compute".to_string()],
            quota_command: vec!["projinfo".to_string()],
        },
        preflight: ContainerPreflightPolicy {
            minimum_available_bytes: 1,
            minimum_available_inodes: 1,
            maximum_clock_skew_seconds: 30,
            required_commands: vec!["git".to_string()],
            required_contracts: vec!["containers/contracts/governance.yml".to_string()],
        },
    };
    validate_container_governance(&governance)?;
    Ok((governance, commit))
}

fn request(
    governance: &ContainerGovernance,
    commit: &str,
    scope: RepositoryFreezeScope,
    checkout_paths: Vec<PathBuf>,
) -> RepositoryFreezeRequest {
    RepositoryFreezeRequest {
        governance: governance.clone(),
        hostname: "worker-7".to_string(),
        commit: commit.to_string(),
        scope,
        checkout_paths,
        source_namespace: "source-freezes".to_string(),
    }
}

#[test]
fn repository_freeze_materializes_scoped_clean_worktrees() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (governance, commit) = fixture(root.path())?;

    let contracts = freeze_repository(&request(
        &governance,
        &commit,
        RepositoryFreezeScope::Contracts,
        Vec::new(),
    ))?;
    let complete = freeze_repository(&request(
        &governance,
        &commit,
        RepositoryFreezeScope::Complete,
        Vec::new(),
    ))?;
    let sources = freeze_repository(&request(
        &governance,
        &commit,
        RepositoryFreezeScope::Sources,
        vec![PathBuf::from("other")],
    ))?;

    assert_eq!(
        std::fs::read_to_string(contracts.join("containers/contracts/governance.yml"))?,
        "schema_version: test\n"
    );
    assert!(!contracts.join("other/source.txt").exists());
    assert_eq!(std::fs::read_to_string(complete.join("other/source.txt"))?, "complete tree\n");
    assert_eq!(std::fs::read_to_string(sources.join("other/source.txt"))?, "complete tree\n");
    assert!(!sources.join("containers/contracts/governance.yml").exists());
    assert_eq!(
        complete.parent().and_then(Path::file_name).and_then(|value| value.to_str()),
        Some("image-builds")
    );
    assert_eq!(
        sources.parent().and_then(Path::file_name).and_then(|value| value.to_str()),
        Some("source-freezes")
    );
    for path in [&contracts, &complete, &sources] {
        assert_eq!(git(path, &["rev-parse", "HEAD"])?, commit);
        assert!(git(path, &["status", "--porcelain"])?.is_empty());
    }
    assert_eq!(
        freeze_repository(&request(
            &governance,
            &commit,
            RepositoryFreezeScope::Contracts,
            Vec::new(),
        ))?,
        contracts
    );
    Ok(())
}

#[test]
fn repository_freeze_rejects_unsafe_sources_and_dirty_authority() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (governance, commit) = fixture(root.path())?;
    let unsafe_request = request(
        &governance,
        &commit,
        RepositoryFreezeScope::Sources,
        vec![PathBuf::from("../outside")],
    );
    assert!(freeze_repository(&unsafe_request).is_err());

    std::fs::write(governance.paths.active_repository.join("untracked.txt"), "dirty\n")?;
    let error = freeze_repository(&request(
        &governance,
        &commit,
        RepositoryFreezeScope::Contracts,
        Vec::new(),
    ))
    .err()
    .context("dirty source repository was accepted")?;
    assert!(format!("{error:#}").contains("must be clean"));
    Ok(())
}

#[test]
fn checked_site_governance_loads_through_container_policy() -> Result<()> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let policy =
        load_container_governance(&repository.join("containers/contracts/sites/lunarc.yml"))?;
    assert_eq!(policy.schema_version, CONTAINER_GOVERNANCE_SCHEMA_V1);
    assert_eq!(policy.site.id, "lunarc");
    Ok(())
}
