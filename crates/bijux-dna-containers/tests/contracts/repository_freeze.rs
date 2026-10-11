use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};
use bijux_dna_containers::api::{
    container_governance_schema, freeze_repository, load_container_governance,
    validate_container_governance, validate_container_governance_host,
    write_container_governance_schema, ContainerGovernance, ContainerPathPolicy,
    ContainerPreflightPolicy, ContainerRuntimePolicy, ContainerSitePolicy, ContainerSlurmPolicy,
    RepositoryFreezeRequest, RepositoryFreezeScope, CONTAINER_GOVERNANCE_SCHEMA_V1,
};

#[test]
fn tracked_governance_schema_matches_rust_authority() -> Result<()> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tracked = std::fs::read_to_string(
        repository.join("containers/contracts/config-governance.schema.json"),
    )?;
    let checked: serde_json::Value = serde_json::from_str(&tracked)?;
    assert_eq!(checked, container_governance_schema());
    let generated = tempfile::tempdir()?;
    let output = generated.path().join("config-governance.schema.json");
    write_container_governance_schema(&output)?;
    assert_eq!(std::fs::read_to_string(output)?, tracked);
    Ok(())
}

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
fn repository_freeze_materializes_scoped_self_contained_snapshots() -> Result<()> {
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
        contracts.parent().and_then(Path::file_name).and_then(|value| value.to_str()),
        Some("contract-snapshots")
    );
    assert_eq!(complete.parent(), Some(governance.paths.frozen_repositories.as_path()));
    assert_eq!(
        sources.parent().and_then(Path::file_name).and_then(|value| value.to_str()),
        Some("source-freezes")
    );
    for path in [&contracts, &complete, &sources] {
        assert_eq!(git(path, &["rev-parse", "HEAD"])?, commit);
        assert!(git(path, &["status", "--porcelain"])?.is_empty());
        assert!(path.join(".git").is_dir());
        assert!(!path.join(".git/objects/info/alternates").exists());
        assert!(git(path, &["remote"])?.is_empty());
        assert!(walkdir::WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| !entry.file_type().is_symlink())
            .all(|entry| entry
                .metadata()
                .is_ok_and(|metadata| { metadata.permissions().mode() & 0o222 == 0 })));
    }
    let registered_worktrees = git(&governance.paths.active_repository, &["worktree", "list"])?;
    assert_eq!(registered_worktrees.lines().count(), 1);
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
    let study_root = Path::new("/lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY");
    assert_eq!(policy.paths.study_root, study_root);
    assert_eq!(policy.paths.containers_root, study_root.join("containers"));
    assert_eq!(policy.paths.data_root, study_root.join("data"));
    assert_eq!(policy.paths.results_root, study_root.join("results"));
    assert_eq!(policy.paths.finalization_root, study_root.join("evidence/finalization"));
    assert_eq!(policy.paths.active_repository, Path::new("/home/bijan/bijux/bijux-genomics"));
    assert_eq!(policy.paths.frozen_repositories, study_root.join("data/evidence/source-checkouts"));
    for hostname in ["cosmos1", "cosmos1.int.lunarc", "cn076", "cn076.int", "cn076.int.lunarc"] {
        validate_container_governance_host(&policy, hostname)?;
    }
    for hostname in ["cosmos1.int", "cn076.example.org", "login1", "cn.int"] {
        assert!(validate_container_governance_host(&policy, hostname).is_err());
    }
    Ok(())
}
