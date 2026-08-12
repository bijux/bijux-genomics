use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{bail, ensure, Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::model::{
    RepositoryFreezeArchiveReceipt, RepositoryFreezeArchiveRequest,
    REPOSITORY_FREEZE_ARCHIVE_RECEIPT_SCHEMA_V1,
};
use super::validation::validate_container_governance_host;

const ARCHIVE_DIRECTORY: &str = "archives";

/// Create a checksum-bound Git bundle and prove that it restores the frozen snapshot exactly.
///
/// The source snapshot is never removed by this operation. Alternate-backed historical snapshots
/// are accepted only while their referenced objects remain readable; the restoration proof must
/// produce a self-contained checkout before the archive is published.
///
/// # Errors
/// Returns an error when the request escapes the governed root, repository identity differs, the
/// bundle is incomplete, or a self-contained restoration cannot be proven.
pub fn archive_frozen_repository(
    request: &RepositoryFreezeArchiveRequest,
) -> Result<RepositoryFreezeArchiveReceipt> {
    let relative = validate_archive_request(request, true)?;
    let archive_root = request.governance.paths.frozen_repositories.join(ARCHIVE_DIRECTORY);
    if archive_root.symlink_metadata().is_err() {
        std::fs::create_dir_all(&archive_root).with_context(|| {
            format!("create repository archive root {}", archive_root.display())
        })?;
    }
    ensure!(archive_root.is_dir(), "repository archive root is not a directory");
    ensure!(
        !archive_root.symlink_metadata()?.file_type().is_symlink(),
        "repository archive root cannot be a symlink"
    );
    ensure!(
        archive_root
            .canonicalize()?
            .starts_with(request.governance.paths.frozen_repositories.canonicalize()?),
        "repository archive root escaped the governed root"
    );
    let identity = archive_identity(&relative, &request.commit);
    let bundle_path = archive_root.join(format!("{identity}.bundle"));
    let receipt_path = archive_root.join(format!("{identity}.receipt.json"));
    let bundle_exists = bundle_path.is_file();
    let receipt_exists = receipt_path.is_file();
    ensure!(
        bundle_exists == receipt_exists,
        "repository archive publication is incomplete for `{identity}`"
    );
    if receipt_exists {
        let receipt = load_receipt(&receipt_path)?;
        verify_frozen_repository_archive(request, &receipt)?;
        return Ok(receipt);
    }

    let sparse_checkout_paths = sparse_checkout_paths(&request.snapshot)?;
    let tracked_index_sha256 = sha256_text(&git_text(&request.snapshot, &["ls-files", "-s"])?);
    let partial_bundle = archive_root.join(format!(".{identity}.bundle.partial"));
    ensure!(partial_bundle.symlink_metadata().is_err(), "repository archive staging bundle exists");
    let archive_result = (|| {
        let output = Command::new("git")
            .arg("-C")
            .arg(&request.snapshot)
            .args(["bundle", "create"])
            .arg(&partial_bundle)
            .arg("HEAD")
            .output()
            .context("create frozen repository bundle")?;
        ensure_command_succeeded(&output, "git bundle create")?;
        verify_bundle(&request.snapshot, &partial_bundle)?;
        prove_restoration(
            &archive_root,
            &identity,
            &partial_bundle,
            &request.commit,
            &sparse_checkout_paths,
            &tracked_index_sha256,
        )
    })();
    if let Err(error) = archive_result {
        let cleanup = remove_file_if_present(&partial_bundle);
        return Err(error).context(if cleanup {
            "archive frozen repository"
        } else {
            "archive frozen repository; staging cleanup failed"
        });
    }
    std::fs::rename(&partial_bundle, &bundle_path)
        .with_context(|| format!("publish frozen repository bundle {}", bundle_path.display()))?;
    make_path_read_only(&bundle_path)?;

    let receipt = RepositoryFreezeArchiveReceipt {
        schema_version: REPOSITORY_FREEZE_ARCHIVE_RECEIPT_SCHEMA_V1.to_string(),
        snapshot_relative_path: relative,
        commit: request.commit.clone(),
        bundle_path: bundle_path.clone(),
        bundle_sha256: sha256_file(&bundle_path)?,
        tracked_index_sha256,
        sparse_checkout_paths,
        restoration_verified: true,
    };
    let receipt_bytes: Vec<u8> =
        serde_json::to_vec_pretty(&receipt)?.into_iter().chain([b'\n']).collect();
    let partial_receipt = archive_root.join(format!(".{identity}.receipt.partial"));
    ensure!(
        partial_receipt.symlink_metadata().is_err(),
        "repository archive receipt staging path exists"
    );
    std::fs::write(&partial_receipt, receipt_bytes).with_context(|| {
        format!("write repository archive receipt {}", partial_receipt.display())
    })?;
    std::fs::rename(&partial_receipt, &receipt_path).with_context(|| {
        format!("publish repository archive receipt {}", receipt_path.display())
    })?;
    make_path_read_only(&receipt_path)?;
    Ok(receipt)
}

/// Verify an archive receipt, bundle checksum, source identity, and restoration feasibility.
///
/// # Errors
/// Returns an error when any bound identity differs or restoration is not self-contained.
pub fn verify_frozen_repository_archive(
    request: &RepositoryFreezeArchiveRequest,
    receipt: &RepositoryFreezeArchiveReceipt,
) -> Result<()> {
    let relative = validate_archive_request(request, false)?;
    ensure!(
        receipt.schema_version == REPOSITORY_FREEZE_ARCHIVE_RECEIPT_SCHEMA_V1,
        "unsupported repository freeze archive receipt schema"
    );
    ensure!(
        receipt.snapshot_relative_path == relative,
        "repository archive snapshot identity drifted"
    );
    ensure!(receipt.commit == request.commit, "repository archive commit identity drifted");
    ensure!(receipt.restoration_verified, "repository archive lacks restoration proof");
    let archive_root = request.governance.paths.frozen_repositories.join(ARCHIVE_DIRECTORY);
    ensure!(
        receipt.bundle_path.parent() == Some(archive_root.as_path()),
        "repository archive bundle escaped governed root"
    );
    ensure!(
        sha256_file(&receipt.bundle_path)? == receipt.bundle_sha256,
        "repository archive checksum drifted"
    );
    if request.snapshot.is_dir() {
        ensure!(
            sha256_text(&git_text(&request.snapshot, &["ls-files", "-s"])?)
                == receipt.tracked_index_sha256,
            "repository archive source index drifted"
        );
    }
    let verification_repository = if request.snapshot.is_dir() {
        &request.snapshot
    } else {
        &request.governance.paths.active_repository
    };
    verify_bundle(verification_repository, &receipt.bundle_path)?;
    let identity = archive_identity(&relative, &request.commit);
    prove_restoration(
        &archive_root,
        &identity,
        &receipt.bundle_path,
        &request.commit,
        &receipt.sparse_checkout_paths,
        &receipt.tracked_index_sha256,
    )
}

/// Remove a frozen checkout only after its archive and restoration proof validate.
///
/// # Errors
/// Returns an error when archive verification fails or the exact governed snapshot cannot be
/// retired. A missing snapshot is treated as an idempotent completed retirement.
pub fn retire_frozen_repository(
    request: &RepositoryFreezeArchiveRequest,
    receipt: &RepositoryFreezeArchiveReceipt,
) -> Result<()> {
    verify_frozen_repository_archive(request, receipt)?;
    if request.snapshot.symlink_metadata().is_err() {
        return Ok(());
    }
    let parent = request.snapshot.parent().context("frozen repository snapshot has no parent")?;
    let name =
        request.snapshot.file_name().context("frozen repository snapshot has no file name")?;
    let identity = archive_identity(&receipt.snapshot_relative_path, &receipt.commit);
    let retiring = parent.join(format!(".{}.retiring-{identity}", name.to_string_lossy()));
    ensure!(retiring.symlink_metadata().is_err(), "repository retirement staging path exists");
    std::fs::rename(&request.snapshot, &retiring).with_context(|| {
        format!("stage frozen repository retirement {}", request.snapshot.display())
    })?;
    if remove_tree(&retiring) {
        return Ok(());
    }
    match restore_frozen_repository_archive(request, receipt) {
        Ok(_) => bail!(
            "retire frozen repository {} failed after staging; archive rollback restored the governed path",
            request.snapshot.display()
        ),
        Err(rollback_error) => Err(rollback_error).context(format!(
            "retire frozen repository {} failed after staging and archive rollback also failed",
            request.snapshot.display()
        )),
    }
}

/// Restore an archived repository to its original governed path atomically.
///
/// # Errors
/// Returns an error when the archive is invalid, the destination already exists with a different
/// identity, or a self-contained read-only checkout cannot be restored.
pub fn restore_frozen_repository_archive(
    request: &RepositoryFreezeArchiveRequest,
    receipt: &RepositoryFreezeArchiveReceipt,
) -> Result<PathBuf> {
    verify_frozen_repository_archive(request, receipt)?;
    if request.snapshot.is_dir() {
        super::repository_freeze::verify_frozen_repository(&request.snapshot, &request.commit)?;
        return Ok(request.snapshot.clone());
    }
    let parent = request.snapshot.parent().context("frozen repository snapshot has no parent")?;
    std::fs::create_dir_all(parent)?;
    let name =
        request.snapshot.file_name().context("frozen repository snapshot has no file name")?;
    let staging = parent.join(format!(".{}.restore-partial", name.to_string_lossy()));
    ensure!(staging.symlink_metadata().is_err(), "repository restore staging path exists");
    let restore_result = (|| {
        restore_bundle(
            &receipt.bundle_path,
            &staging,
            &request.commit,
            &receipt.sparse_checkout_paths,
        )?;
        ensure!(
            sha256_text(&git_text(&staging, &["ls-files", "-s"])?) == receipt.tracked_index_sha256,
            "restored repository index drifted"
        );
        make_tree_read_only(&staging)?;
        super::repository_freeze::verify_frozen_repository(&staging, &request.commit)
    })();
    if let Err(error) = restore_result {
        let cleanup = remove_tree(&staging);
        return Err(error).context(if cleanup {
            "restore frozen repository archive"
        } else {
            "restore frozen repository archive; staging cleanup failed"
        });
    }
    std::fs::rename(&staging, &request.snapshot)
        .with_context(|| format!("publish restored repository {}", request.snapshot.display()))?;
    Ok(request.snapshot.clone())
}

fn validate_archive_request(
    request: &RepositoryFreezeArchiveRequest,
    require_snapshot: bool,
) -> Result<PathBuf> {
    validate_container_governance_host(&request.governance, &request.hostname)?;
    ensure!(
        Regex::new(r"^[0-9a-f]{40}$")?.is_match(&request.commit),
        "archive commit must be a full Git commit"
    );
    let root = &request.governance.paths.frozen_repositories;
    ensure!(root.is_dir(), "frozen repository root does not exist");
    let canonical_root = root.canonicalize()?;
    ensure!(request.snapshot.starts_with(root), "frozen repository snapshot escaped governed root");
    let relative = request.snapshot.strip_prefix(root)?.to_path_buf();
    ensure!(!relative.as_os_str().is_empty(), "cannot archive the frozen repository root");
    ensure!(
        relative.components().all(|component| matches!(component, Component::Normal(_))),
        "frozen repository snapshot contains unsafe path components"
    );
    ensure!(!relative.starts_with(ARCHIVE_DIRECTORY), "cannot archive the repository archive root");
    if require_snapshot || request.snapshot.exists() {
        ensure!(request.snapshot.is_dir(), "frozen repository snapshot does not exist");
        ensure!(
            !request.snapshot.symlink_metadata()?.file_type().is_symlink(),
            "frozen repository snapshot cannot be a symlink"
        );
        ensure!(
            request.snapshot.canonicalize()?.starts_with(&canonical_root),
            "frozen repository snapshot resolved outside the governed root"
        );
        ensure!(
            git_text(&request.snapshot, &["rev-parse", "HEAD"])? == request.commit,
            "frozen repository commit identity drifted"
        );
        ensure!(
            git_text(&request.snapshot, &["status", "--porcelain"])?.is_empty(),
            "frozen repository snapshot is dirty"
        );
    } else {
        let parent =
            request.snapshot.parent().context("frozen repository snapshot has no parent")?;
        ensure!(parent.is_dir(), "frozen repository snapshot parent does not exist");
        ensure!(
            parent.canonicalize()?.starts_with(&canonical_root),
            "frozen repository snapshot parent resolved outside the governed root"
        );
    }
    Ok(relative)
}

fn archive_identity(relative: &Path, commit: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(relative.to_string_lossy().as_bytes());
    digest.update([0]);
    digest.update(commit.as_bytes());
    let encoded = hex_bytes(&digest.finalize());
    format!("{}-{}", &commit[..12], &encoded[..12])
}

fn sparse_checkout_paths(repository: &Path) -> Result<Vec<PathBuf>> {
    let enabled =
        git_text(repository, &["config", "--bool", "core.sparseCheckout"]).unwrap_or_default();
    if enabled != "true" {
        return Ok(Vec::new());
    }
    let output = git_text(repository, &["sparse-checkout", "list"])?;
    Ok(output.lines().filter(|line| !line.is_empty()).map(PathBuf::from).collect())
}

fn prove_restoration(
    archive_root: &Path,
    identity: &str,
    bundle: &Path,
    commit: &str,
    sparse_paths: &[PathBuf],
    tracked_index_sha256: &str,
) -> Result<()> {
    let proof = archive_root.join(format!(".{identity}.restore-proof"));
    ensure!(proof.symlink_metadata().is_err(), "repository restoration proof path exists");
    let result = (|| {
        restore_bundle(bundle, &proof, commit, sparse_paths)?;
        ensure!(
            !proof.join(".git/objects/info/alternates").exists(),
            "restored repository depends on alternate objects"
        );
        ensure!(
            git_text(&proof, &["rev-parse", "HEAD"])? == commit,
            "restored repository commit drifted"
        );
        ensure!(
            git_text(&proof, &["status", "--porcelain"])?.is_empty(),
            "restored repository is dirty"
        );
        ensure!(
            sha256_text(&git_text(&proof, &["ls-files", "-s"])?) == tracked_index_sha256,
            "restored repository index drifted"
        );
        Ok(())
    })();
    let cleanup = remove_tree(&proof);
    result.with_context(|| {
        if cleanup {
            "prove frozen repository restoration"
        } else {
            "prove frozen repository restoration; cleanup failed"
        }
    })
}

fn restore_bundle(
    bundle: &Path,
    target: &Path,
    commit: &str,
    sparse_paths: &[PathBuf],
) -> Result<()> {
    let output = Command::new("git")
        .args(["clone", "--no-hardlinks", "--no-checkout", "--quiet"])
        .arg(bundle)
        .arg(target)
        .output()
        .context("clone frozen repository archive")?;
    ensure_command_succeeded(&output, "git clone archive")?;
    if !sparse_paths.is_empty() {
        let mut arguments =
            vec!["sparse-checkout".to_string(), "set".to_string(), "--cone".to_string()];
        arguments.extend(sparse_paths.iter().map(|path| path.display().to_string()));
        git(target, &arguments)?;
    }
    git(target, &["checkout".to_string(), "--detach".to_string(), commit.to_string()])?;
    git(target, &["remote".to_string(), "remove".to_string(), "origin".to_string()])?;
    Ok(())
}

fn verify_bundle(repository: &Path, bundle: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["bundle", "verify"])
        .arg(bundle)
        .output()?;
    ensure_command_succeeded(&output, "git bundle verify")
}

fn load_receipt(path: &Path) -> Result<RepositoryFreezeArchiveReceipt> {
    let raw = std::fs::read_to_string(path)?;
    serde_json::from_str(&raw).context("parse repository archive receipt")
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).with_context(|| format!("read {}", path.display()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(hex_bytes(&digest.finalize()))
}

fn sha256_text(value: &str) -> String {
    hex_bytes(&Sha256::digest(value.as_bytes()))
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

fn git_text(repository: &Path, arguments: &[&str]) -> Result<String> {
    let arguments = arguments.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let output = git(repository, &arguments)?;
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn git(repository: &Path, arguments: &[String]) -> Result<Output> {
    let output = Command::new("git").arg("-C").arg(repository).args(arguments).output()?;
    ensure_command_succeeded(&output, &format!("git {}", arguments.join(" ")))?;
    Ok(output)
}

fn ensure_command_succeeded(output: &Output, action: &str) -> Result<()> {
    if !output.status.success() {
        bail!("{action} failed: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}

fn make_path_read_only(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(permissions.mode() & !0o222);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

fn make_tree_read_only(path: &Path) -> Result<()> {
    for entry in WalkDir::new(path).contents_first(true).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_symlink() {
            make_path_read_only(entry.path())?;
        }
    }
    Ok(())
}

fn make_tree_owner_writable(path: &Path) -> Result<()> {
    for entry in WalkDir::new(path).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            continue;
        }
        let metadata = entry.metadata()?;
        let mut permissions = metadata.permissions();
        permissions
            .set_mode(permissions.mode() | if entry.file_type().is_dir() { 0o700 } else { 0o200 });
        std::fs::set_permissions(entry.path(), permissions)?;
    }
    Ok(())
}

fn remove_tree(path: &Path) -> bool {
    if path.symlink_metadata().is_err() {
        return true;
    }
    if make_tree_owner_writable(path).is_err() {
        return false;
    }
    std::fs::remove_dir_all(path).is_ok()
}

fn remove_file_if_present(path: &Path) -> bool {
    if path.symlink_metadata().is_err() {
        return true;
    }
    let _ = make_path_owner_writable(path);
    std::fs::remove_file(path).is_ok()
}

fn make_path_owner_writable(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(permissions.mode() | 0o200);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use anyhow::{ensure, Context, Result};

    use super::verify_bundle;

    #[test]
    fn bundle_verification_uses_explicit_repository_context() -> Result<()> {
        let root = tempfile::tempdir()?;
        let repository = root.path().join("repository");
        std::fs::create_dir(&repository)?;
        git(&repository, &["init", "--quiet"])?;
        git(&repository, &["config", "user.name", "Bijux Test"])?;
        git(&repository, &["config", "user.email", "test@bijux.invalid"])?;
        std::fs::write(repository.join("source.txt"), "source\n")?;
        git(&repository, &["add", "source.txt"])?;
        git(&repository, &["commit", "--quiet", "-m", "fixture"])?;
        let bundle = root.path().join("repository.bundle");
        let bundle_argument = bundle.to_str().context("bundle path is not UTF-8")?;
        git(&repository, &["bundle", "create", bundle_argument, "HEAD"])?;

        let external = root.path().join("external-working-directory");
        std::fs::create_dir(&external)?;
        let ambient = Command::new("git")
            .args(["bundle", "verify"])
            .arg(&bundle)
            .current_dir(&external)
            .output()?;
        ensure!(!ambient.status.success(), "ambient non-repository unexpectedly verified bundle");
        verify_bundle(&repository, &bundle)
    }

    fn git(repository: &Path, arguments: &[&str]) -> Result<()> {
        let output = Command::new("git").arg("-C").arg(repository).args(arguments).output()?;
        ensure!(
            output.status.success(),
            "git {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}
