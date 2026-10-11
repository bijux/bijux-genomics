use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{bail, ensure, Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};

use super::model::{RepositoryFreezeRequest, RepositoryFreezeScope};
use super::validation::{safe_checkout_path, validate_container_governance_host};

const CONTRACT_CHECKOUT_PATHS: [&str; 3] =
    ["benchmarks/contracts", "benchmarks/execution", "containers/contracts"];

/// Materialize or verify an immutable, self-contained repository snapshot under the governed root.
///
/// # Errors
/// Returns an error when policy or host admission fails, the source repository is dirty, the
/// commit is unavailable, a requested path is unsafe, or Git cannot create and verify the
/// snapshot.
pub fn freeze_repository(request: &RepositoryFreezeRequest) -> Result<PathBuf> {
    validate_request(request)?;
    let active = &request.governance.paths.active_repository;
    ensure!(active.is_dir(), "active repository does not exist: {}", active.display());
    ensure!(
        git_text(active, &["status", "--porcelain"])?.is_empty(),
        "active repository must be clean before freezing"
    );
    git_text(active, &["cat-file", "-e", &format!("{}^{{commit}}", request.commit)])?;

    let checkout_paths = normalized_checkout_paths(request)?;
    let target = freeze_target(request, &checkout_paths);
    if target.symlink_metadata().is_ok() {
        verify_existing_freeze(&target, &request.commit)?;
        return Ok(target);
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create frozen repository parent {}", parent.display()))?;
    }
    create_snapshot(request, &target, &checkout_paths)?;
    if let Err(error) = verify_existing_freeze(&target, &request.commit) {
        let cleanup = remove_incomplete_snapshot(&target);
        return Err(error).context(format!(
            "verify frozen repository snapshot{}",
            if cleanup { "" } else { "; cleanup failed" }
        ));
    }
    Ok(target)
}

fn validate_request(request: &RepositoryFreezeRequest) -> Result<()> {
    validate_container_governance_host(&request.governance, &request.hostname)?;
    ensure!(
        Regex::new(r"^[0-9a-f]{40}$")?.is_match(&request.commit),
        "frozen repository commit must be a full Git commit"
    );
    match request.scope {
        RepositoryFreezeScope::Sources => {
            ensure!(
                Regex::new(r"^[a-z][a-z0-9-]*$")?.is_match(&request.source_namespace),
                "source freeze namespace must be a durable path identity"
            );
            ensure!(
                !request.checkout_paths.is_empty(),
                "source freeze requires at least one checkout path"
            );
            for path in &request.checkout_paths {
                safe_checkout_path(path)?;
            }
        }
        RepositoryFreezeScope::Contracts | RepositoryFreezeScope::Complete => {
            ensure!(
                request.checkout_paths.is_empty(),
                "checkout paths are valid only for a source freeze"
            );
            ensure!(
                request.source_namespace == "source-freezes",
                "source freeze namespace is valid only for a source freeze"
            );
        }
    }
    Ok(())
}

fn normalized_checkout_paths(request: &RepositoryFreezeRequest) -> Result<Vec<PathBuf>> {
    match request.scope {
        RepositoryFreezeScope::Contracts => {
            Ok(CONTRACT_CHECKOUT_PATHS.iter().map(PathBuf::from).collect())
        }
        RepositoryFreezeScope::Sources => request
            .checkout_paths
            .iter()
            .map(|path| safe_checkout_path(path))
            .collect::<Result<BTreeSet<_>>>()
            .map(|paths| paths.into_iter().collect()),
        RepositoryFreezeScope::Complete => Ok(Vec::new()),
    }
}

fn freeze_target(request: &RepositoryFreezeRequest, checkout_paths: &[PathBuf]) -> PathBuf {
    let root = &request.governance.paths.frozen_repositories;
    match request.scope {
        RepositoryFreezeScope::Contracts => root.join(&request.commit),
        RepositoryFreezeScope::Sources => {
            let mut digest = Sha256::new();
            for (index, path) in checkout_paths.iter().enumerate() {
                if index > 0 {
                    digest.update([0]);
                }
                digest.update(path.to_string_lossy().as_bytes());
            }
            let identity = hex_bytes(&digest.finalize());
            root.join(&request.source_namespace).join(format!(
                "{}-{}",
                request.commit,
                &identity[..12]
            ))
        }
        RepositoryFreezeScope::Complete => root.join("image-builds").join(&request.commit),
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn create_snapshot(
    request: &RepositoryFreezeRequest,
    target: &Path,
    checkout_paths: &[PathBuf],
) -> Result<()> {
    let active = &request.governance.paths.active_repository;
    let name = target.file_name().context("frozen repository target has no file name")?;
    let staging = target.with_file_name(format!(
        ".{}.partial-{}",
        name.to_string_lossy(),
        std::process::id()
    ));
    ensure!(staging.symlink_metadata().is_err(), "frozen repository staging path exists");
    if let Err(error) = clone_repository(active, &staging) {
        let cleanup = remove_incomplete_snapshot(&staging);
        return Err(error).context(format!(
            "clone frozen repository snapshot{}",
            if cleanup { "" } else { "; incomplete snapshot remains" }
        ));
    }
    if let Err(error) = configure_snapshot(request, &staging, checkout_paths) {
        let cleanup = remove_incomplete_snapshot(&staging);
        return Err(error).context(format!(
            "configure frozen repository snapshot{}",
            if cleanup { "" } else { "; incomplete snapshot remains" }
        ));
    }
    if let Err(error) = verify_existing_freeze(&staging, &request.commit) {
        let cleanup = remove_incomplete_snapshot(&staging);
        return Err(error).context(format!(
            "verify frozen repository snapshot before publication{}",
            if cleanup { "" } else { "; incomplete snapshot remains" }
        ));
    }
    if let Err(error) = std::fs::rename(&staging, target) {
        let cleanup = remove_incomplete_snapshot(&staging);
        return Err(error).with_context(|| {
            format!(
                "publish frozen repository snapshot {}{}",
                target.display(),
                if cleanup { "" } else { "; incomplete snapshot remains" }
            )
        });
    }
    Ok(())
}

fn clone_repository(active: &Path, staging: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["clone", "--no-hardlinks", "--no-checkout", "--quiet"])
        .arg(active)
        .arg(staging)
        .output()
        .context("clone self-contained frozen repository")?;
    if !output.status.success() {
        bail!("git clone failed: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}

fn configure_snapshot(
    request: &RepositoryFreezeRequest,
    staging: &Path,
    checkout_paths: &[PathBuf],
) -> Result<()> {
    if request.scope != RepositoryFreezeScope::Complete {
        let mut sparse =
            vec!["sparse-checkout".to_string(), "set".to_string(), "--cone".to_string()];
        sparse.extend(checkout_paths.iter().map(|path| path.display().to_string()));
        git(staging, &sparse)?;
    }
    git(staging, &["checkout".to_string(), "--detach".to_string(), request.commit.clone()])?;
    git(staging, &["remote".to_string(), "remove".to_string(), "origin".to_string()])?;
    Ok(())
}

fn verify_existing_freeze(target: &Path, commit: &str) -> Result<()> {
    ensure!(
        git_text(target, &["rev-parse", "HEAD"])? == commit,
        "existing frozen repository snapshot has the wrong commit: {}",
        target.display()
    );
    ensure!(
        git_text(target, &["status", "--porcelain"])?.is_empty(),
        "existing frozen repository snapshot is dirty: {}",
        target.display()
    );
    ensure!(target.join(".git").is_dir(), "frozen repository metadata is not self-contained");
    ensure!(
        !target.join(".git/objects/info/alternates").exists(),
        "frozen repository snapshot depends on alternate object storage"
    );
    Ok(())
}

fn git_text(repository: &Path, arguments: &[&str]) -> Result<String> {
    let arguments = arguments.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let output = git(repository, &arguments)?;
    String::from_utf8(output.stdout)
        .context("Git output is not UTF-8")
        .map(|value| value.trim().to_string())
}

fn git(repository: &Path, arguments: &[String]) -> Result<Output> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .output()
        .with_context(|| format!("run git {}", arguments.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output)
}

fn remove_incomplete_snapshot(target: &Path) -> bool {
    target.symlink_metadata().is_err() || std::fs::remove_dir_all(target).is_ok()
}
