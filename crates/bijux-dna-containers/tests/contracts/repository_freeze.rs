use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};
use bijux_dna_containers::api::{
    archive_and_retire_frozen_repository, archive_frozen_repository, container_governance_schema,
    freeze_repository, load_container_governance, repository_freeze_archive_receipt_schema,
    restore_frozen_repository_archive, retire_frozen_repository, validate_container_governance,
    validate_container_governance_host, verify_frozen_repository, verify_frozen_repository_archive,
    write_container_governance_schema, write_repository_freeze_archive_receipt_schema,
    ContainerGovernance, ContainerPathPolicy, ContainerPreflightPolicy, ContainerRuntimePolicy,
    ContainerSitePolicy, ContainerSlurmPolicy, RepositoryFreezeArchiveRequest,
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

#[test]
fn tracked_repository_freeze_archive_schema_matches_rust_authority() -> Result<()> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tracked = std::fs::read_to_string(
        repository.join("containers/contracts/repository-freeze-archive-receipt.schema.json"),
    )?;
    let tracked_value: serde_json::Value = serde_json::from_str(&tracked)?;
    let generated_value = serde_json::to_value(repository_freeze_archive_receipt_schema())?;
    assert_eq!(tracked_value, generated_value);
    let generated = tempfile::tempdir()?;
    let output = generated.path().join("repository-freeze-archive-receipt.schema.json");
    write_repository_freeze_archive_receipt_schema(&output)?;
    let written: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(output)?)?;
    assert_eq!(written, tracked_value);
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
        verify_frozen_repository(path, &commit)?;
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

    let mut permissions = std::fs::metadata(&complete)?.permissions();
    permissions.set_mode(permissions.mode() | 0o200);
    std::fs::set_permissions(&complete, permissions)?;
    let error = verify_frozen_repository(&complete, &commit)
        .err()
        .context("writable repository snapshot was accepted")?;
    assert!(format!("{error:#}").contains("remains writable"));
    freeze_repository(&request(&governance, &commit, RepositoryFreezeScope::Complete, Vec::new()))?;
    verify_frozen_repository(&complete, &commit)?;

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
fn repository_freeze_rejects_insufficient_governed_storage_before_cloning() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (mut governance, commit) = fixture(root.path())?;
    let target = governance.paths.frozen_repositories.join(&commit);

    governance.preflight.minimum_available_bytes = u64::MAX;
    let error = freeze_repository(&request(
        &governance,
        &commit,
        RepositoryFreezeScope::Complete,
        Vec::new(),
    ))
    .err()
    .context("repository freeze ignored its byte reserve")?;
    assert!(format!("{error:#}").contains("available bytes"));
    assert!(!target.exists());

    governance.preflight.minimum_available_bytes = 1;
    governance.preflight.minimum_available_inodes = u64::MAX;
    let error = freeze_repository(&request(
        &governance,
        &commit,
        RepositoryFreezeScope::Complete,
        Vec::new(),
    ))
    .err()
    .context("repository freeze ignored its inode reserve")?;
    assert!(format!("{error:#}").contains("available inodes"));
    assert!(!target.exists());
    Ok(())
}

#[test]
fn repository_freeze_archive_proves_retirement_and_exact_restoration() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (governance, commit) = fixture(root.path())?;
    let snapshot = freeze_repository(&request(
        &governance,
        &commit,
        RepositoryFreezeScope::Sources,
        vec![PathBuf::from("other")],
    ))?;
    let archive_request = RepositoryFreezeArchiveRequest {
        governance,
        hostname: "worker-7".to_string(),
        snapshot: snapshot.clone(),
        commit: commit.clone(),
    };

    let receipt = archive_frozen_repository(&archive_request)?;
    assert!(receipt.restoration_verified);
    assert_eq!(receipt.sparse_checkout_paths, [PathBuf::from("other")]);
    assert!(receipt.bundle_path.is_file());
    verify_frozen_repository_archive(&archive_request, &receipt)?;

    retire_frozen_repository(&archive_request, &receipt)?;
    assert!(!snapshot.exists());
    retire_frozen_repository(&archive_request, &receipt)?;
    let restored = restore_frozen_repository_archive(&archive_request, &receipt)?;
    assert_eq!(restored, snapshot);
    verify_frozen_repository(&restored, &commit)?;
    assert_eq!(std::fs::read_to_string(restored.join("other/source.txt"))?, "complete tree\n");
    assert!(!restored.join("containers/contracts/governance.yml").exists());
    Ok(())
}

#[test]
fn repository_archive_and_retire_closes_the_storage_lifecycle() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (governance, commit) = fixture(root.path())?;
    let snapshot = freeze_repository(&request(
        &governance,
        &commit,
        RepositoryFreezeScope::Sources,
        vec![PathBuf::from("other")],
    ))?;
    let archive_request = RepositoryFreezeArchiveRequest {
        governance,
        hostname: "worker-7".to_string(),
        snapshot: snapshot.clone(),
        commit: commit.clone(),
    };

    let receipt = archive_and_retire_frozen_repository(&archive_request)?;
    assert!(receipt.restoration_verified);
    assert!(receipt.bundle_path.is_file());
    assert!(!snapshot.exists());
    verify_frozen_repository_archive(&archive_request, &receipt)?;
    assert_eq!(archive_and_retire_frozen_repository(&archive_request)?, receipt);

    let restored = restore_frozen_repository_archive(&archive_request, &receipt)?;
    verify_frozen_repository(&restored, &commit)?;
    assert_eq!(std::fs::read_to_string(restored.join("other/source.txt"))?, "complete tree\n");
    Ok(())
}

#[test]
fn repository_archive_can_precede_inode_expensive_snapshot_materialization() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (governance, commit) = fixture(root.path())?;
    std::fs::create_dir_all(&governance.paths.frozen_repositories)?;
    let snapshot = governance.paths.frozen_repositories.join(&commit);
    assert!(!snapshot.exists());
    let request = RepositoryFreezeArchiveRequest {
        governance,
        hostname: "worker-7".to_string(),
        snapshot: snapshot.clone(),
        commit: commit.clone(),
    };

    let receipt = archive_frozen_repository(&request)?;
    assert!(!snapshot.exists());
    assert!(receipt.sparse_checkout_paths.is_empty());
    assert!(receipt.restoration_verified);
    verify_frozen_repository_archive(&request, &receipt)?;

    let restored = restore_frozen_repository_archive(&request, &receipt)?;
    assert_eq!(restored, snapshot);
    verify_frozen_repository(&restored, &commit)?;
    assert_eq!(std::fs::read_to_string(restored.join("other/source.txt"))?, "complete tree\n");
    Ok(())
}

#[test]
fn repository_freeze_archive_detaches_historical_alternate_objects() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (governance, commit) = fixture(root.path())?;
    let snapshot = governance.paths.frozen_repositories.join("historical/alternate-backed");
    std::fs::create_dir_all(snapshot.parent().context("historical snapshot parent")?)?;
    let output = Command::new("git")
        .args(["clone", "--shared", "--no-checkout", "--quiet"])
        .arg(&governance.paths.active_repository)
        .arg(&snapshot)
        .output()?;
    ensure!(output.status.success(), "shared clone failed");
    git(&snapshot, &["sparse-checkout", "set", "--cone", "other"])?;
    git(&snapshot, &["checkout", "--detach", &commit])?;
    git(&snapshot, &["remote", "remove", "origin"])?;
    assert!(snapshot.join(".git/objects/info/alternates").is_file());
    for entry in walkdir::WalkDir::new(&snapshot).contents_first(true).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            continue;
        }
        let metadata = entry.metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() & !0o222);
        std::fs::set_permissions(entry.path(), permissions)?;
    }

    let request = RepositoryFreezeArchiveRequest {
        governance,
        hostname: "worker-7".to_string(),
        snapshot: snapshot.clone(),
        commit: commit.clone(),
    };
    let receipt = archive_frozen_repository(&request)?;
    retire_frozen_repository(&request, &receipt)?;
    let restored = restore_frozen_repository_archive(&request, &receipt)?;
    assert!(!restored.join(".git/objects/info/alternates").exists());
    verify_frozen_repository(&restored, &commit)?;
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
