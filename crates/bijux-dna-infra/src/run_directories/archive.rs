use std::fs;
use std::path::{Component, Path};

use crate::{
    atomic_create_json, ensure_dir, hash_bytes_sha256, hash_file_sha256, read_to_end_bounded,
    same_filesystem, FileLock, IoError, IoErrorKind,
};

use super::contract::{
    AttemptArchiveDisposition, AttemptArchiveFileV1, AttemptArchivePaths, AttemptArchiveReceiptV1,
    AttemptArchiveRollbackReceiptV1, AttemptEvidenceOwner, AttemptLayoutIdentity,
    ATTEMPT_ARCHIVE_ROLLBACK_SCHEMA_V1, ATTEMPT_ARCHIVE_SCHEMA_V1,
};
use super::operations::{
    attempt_layout_paths, attempt_lock_path, layout_from_attempt_dir, validate_attempt_layout,
};

const ATTEMPT_ARCHIVES_DIR: &str = ".attempt-archives";
const ATTEMPT_ARCHIVE_ROLLBACKS_DIR: &str = ".attempt-archive-rollbacks";
const ATTEMPT_ARCHIVE_RECEIPT: &str = "attempt-archive.json";
const MAX_ATTEMPT_ARCHIVE_RECEIPT_BYTES: usize = 16 * 1024 * 1024;

/// Derive stable source, archive, and rollback-receipt paths for one governed attempt.
///
/// # Errors
/// Returns an error when the attempt identity or owner root is unsafe.
pub fn attempt_archive_paths(
    owner_root: &Path,
    identity: &AttemptLayoutIdentity,
) -> Result<AttemptArchivePaths, IoError> {
    let source = attempt_layout_paths(owner_root, identity)?;
    let archive_dir = owner_root
        .join(ATTEMPT_ARCHIVES_DIR)
        .join(&identity.run_directory)
        .join(&identity.tool_id)
        .join(&identity.campaign_id)
        .join(&identity.attempt_id);
    let archive_receipt_path = archive_dir
        .join(super::ATTEMPT_LAYOUT_CONTRACT.manifests_dir)
        .join(ATTEMPT_ARCHIVE_RECEIPT);
    let rollback_receipt_path = owner_root
        .join(ATTEMPT_ARCHIVE_ROLLBACKS_DIR)
        .join(&identity.run_directory)
        .join(&identity.tool_id)
        .join(&identity.campaign_id)
        .join(format!("{}.json", identity.attempt_id));
    Ok(AttemptArchivePaths {
        owner_root: owner_root.to_path_buf(),
        source,
        archive_dir,
        archive_receipt_path,
        rollback_receipt_path,
    })
}

/// Return the strict JSON schema for an immutable attempt archive receipt.
#[must_use]
pub fn attempt_archive_schema() -> serde_json::Value {
    archive_transaction_schema(ATTEMPT_ARCHIVE_SCHEMA_V1, false)
}

/// Return the strict JSON schema for an immutable attempt archive rollback receipt.
#[must_use]
pub fn attempt_archive_rollback_schema() -> serde_json::Value {
    archive_transaction_schema(ATTEMPT_ARCHIVE_ROLLBACK_SCHEMA_V1, true)
}

/// Atomically move one failed or superseded attempt into its owner-scoped immutable archive.
///
/// Repeating the same operation recovers a move interrupted before receipt publication and then
/// verifies the complete file inventory. The source and archive must share a filesystem.
///
/// # Errors
/// Returns an error for identity drift, symlinks, changed bytes, cross-filesystem movement,
/// conflicting source/archive states, or a conflicting immutable receipt.
pub fn archive_attempt_layout(
    owner_root: &Path,
    owner: AttemptEvidenceOwner,
    identity: &AttemptLayoutIdentity,
    disposition: AttemptArchiveDisposition,
    archived_at_utc: &str,
) -> Result<AttemptArchivePaths, IoError> {
    validate_timestamp(archived_at_utc, "archive timestamp")?;
    let paths = attempt_archive_paths(owner_root, identity)?;
    let _lock = FileLock::try_acquire(&attempt_lock_path(owner_root, identity))?;
    let source_exists = path_exists(&paths.source.attempt_dir)?;
    let archive_exists = path_exists(&paths.archive_dir)?;
    match (source_exists, archive_exists) {
        (true, true) => {
            return Err(corruption(
                "attempt source and archive both exist; refusing ambiguous archival",
            ));
        }
        (false, false) => return Err(corruption("attempt source and archive are both missing")),
        (false, true) => {
            ensure_archive_receipt(&paths, owner, identity, disposition, archived_at_utc)?;
            verify_attempt_archive_locked(&paths, owner, identity, disposition, archived_at_utc)?;
            return Ok(paths);
        }
        (true, false) => {}
    }

    validate_attempt_layout(&paths.source, owner, identity)?;
    inventory(&paths.source.attempt_dir)?;
    let archive_parent = paths
        .archive_dir
        .parent()
        .ok_or_else(|| corruption("attempt archive has no parent directory"))?;
    ensure_dir(archive_parent)?;
    if !same_filesystem(&paths.source.attempt_dir, archive_parent)? {
        return Err(corruption("attempt source and archive must remain on one filesystem"));
    }
    fs::rename(&paths.source.attempt_dir, &paths.archive_dir).map_err(IoError::from_io)?;
    ensure_archive_receipt(&paths, owner, identity, disposition, archived_at_utc)?;
    verify_attempt_archive_locked(&paths, owner, identity, disposition, archived_at_utc)?;
    Ok(paths)
}

/// Verify one owner-scoped attempt archive and its complete regular-file inventory.
///
/// # Errors
/// Returns an error when the active source still exists, archive identity differs, a symlink or
/// unsupported filesystem entry appears, or any recorded file identity changed.
pub fn verify_attempt_archive(
    owner_root: &Path,
    owner: AttemptEvidenceOwner,
    identity: &AttemptLayoutIdentity,
    disposition: AttemptArchiveDisposition,
    archived_at_utc: &str,
) -> Result<AttemptArchivePaths, IoError> {
    validate_timestamp(archived_at_utc, "archive timestamp")?;
    let paths = attempt_archive_paths(owner_root, identity)?;
    let _lock = FileLock::try_acquire(&attempt_lock_path(owner_root, identity))?;
    verify_attempt_archive_locked(&paths, owner, identity, disposition, archived_at_utc)?;
    Ok(paths)
}

/// Restore one verified attempt archive and publish an immutable rollback receipt.
///
/// A retry completes an interruption after the archive was renamed but before its embedded archive
/// receipt was removed. No path outside this attempt and its dedicated rollback receipt is changed.
///
/// # Errors
/// Returns an error for archive drift, conflicting active state, changed bytes, or an incompatible
/// rollback receipt.
pub fn rollback_attempt_archive(
    owner_root: &Path,
    owner: AttemptEvidenceOwner,
    identity: &AttemptLayoutIdentity,
    disposition: AttemptArchiveDisposition,
    archived_at_utc: &str,
    rolled_back_at_utc: &str,
) -> Result<AttemptArchivePaths, IoError> {
    validate_timestamp(archived_at_utc, "archive timestamp")?;
    validate_timestamp(rolled_back_at_utc, "rollback timestamp")?;
    let paths = attempt_archive_paths(owner_root, identity)?;
    let _lock = FileLock::try_acquire(&attempt_lock_path(owner_root, identity))?;
    let source_receipt = paths.source.manifests_dir.join(ATTEMPT_ARCHIVE_RECEIPT);
    let source_exists = path_exists(&paths.source.attempt_dir)?;
    let archive_exists = path_exists(&paths.archive_dir)?;

    let archive_receipt_bytes = match (source_exists, archive_exists) {
        (false, true) => {
            verify_attempt_archive_locked(&paths, owner, identity, disposition, archived_at_utc)?;
            let bytes = read_to_end_bounded(
                &paths.archive_receipt_path,
                MAX_ATTEMPT_ARCHIVE_RECEIPT_BYTES,
            )?;
            let source_parent = paths
                .source
                .attempt_dir
                .parent()
                .ok_or_else(|| corruption("attempt source has no parent directory"))?;
            ensure_dir(source_parent)?;
            fs::rename(&paths.archive_dir, &paths.source.attempt_dir).map_err(IoError::from_io)?;
            bytes
        }
        (true, false) if path_exists(&source_receipt)? => {
            let bytes = read_to_end_bounded(&source_receipt, MAX_ATTEMPT_ARCHIVE_RECEIPT_BYTES)?;
            let receipt: AttemptArchiveReceiptV1 = serde_json::from_slice(&bytes)
                .map_err(|error| corruption(format!("parse restored archive receipt: {error}")))?;
            validate_receipt_identity(
                &receipt,
                &paths,
                owner,
                identity,
                disposition,
                archived_at_utc,
            )?;
            verify_inventory(&paths.source.attempt_dir, &receipt.files)?;
            bytes
        }
        (true, false) if path_exists(&paths.rollback_receipt_path)? => {
            validate_attempt_layout(&paths.source, owner, identity)?;
            let rollback = verify_rollback_receipt(
                &paths,
                owner,
                identity,
                disposition,
                archived_at_utc,
                rolled_back_at_utc,
            )?;
            verify_inventory(&paths.source.attempt_dir, &rollback.files)?;
            return Ok(paths);
        }
        (true, true) => {
            return Err(corruption(
                "attempt source and archive both exist; refusing ambiguous rollback",
            ));
        }
        _ => return Err(corruption("attempt archive is unavailable for rollback")),
    };

    let archived: AttemptArchiveReceiptV1 = serde_json::from_slice(&archive_receipt_bytes)
        .map_err(|error| corruption(format!("parse archive receipt for rollback: {error}")))?;
    let rollback = AttemptArchiveRollbackReceiptV1 {
        schema_version: ATTEMPT_ARCHIVE_ROLLBACK_SCHEMA_V1.to_string(),
        owner,
        identity: identity.clone(),
        disposition,
        archived_at_utc: archived_at_utc.to_string(),
        rolled_back_at_utc: rolled_back_at_utc.to_string(),
        archive_receipt_sha256: hash_bytes_sha256(&archive_receipt_bytes),
        files: archived.files,
    };
    publish_or_verify_json(&paths.rollback_receipt_path, &rollback)?;
    fs::remove_file(&source_receipt).map_err(IoError::from_io)?;
    validate_attempt_layout(&paths.source, owner, identity)?;
    let persisted = verify_rollback_receipt(
        &paths,
        owner,
        identity,
        disposition,
        archived_at_utc,
        rolled_back_at_utc,
    )?;
    verify_inventory(&paths.source.attempt_dir, &persisted.files)?;
    Ok(paths)
}

fn verify_attempt_archive_locked(
    paths: &AttemptArchivePaths,
    owner: AttemptEvidenceOwner,
    identity: &AttemptLayoutIdentity,
    disposition: AttemptArchiveDisposition,
    archived_at_utc: &str,
) -> Result<(), IoError> {
    if path_exists(&paths.source.attempt_dir)? {
        return Err(corruption("archived attempt remains present at its active source"));
    }
    let archived_layout = layout_from_attempt_dir(&paths.archive_dir);
    validate_attempt_layout(&archived_layout, owner, identity)?;
    let receipt = read_archive_receipt(&paths.archive_receipt_path)?;
    validate_receipt_identity(&receipt, paths, owner, identity, disposition, archived_at_utc)?;
    verify_inventory(&paths.archive_dir, &receipt.files)
}

fn ensure_archive_receipt(
    paths: &AttemptArchivePaths,
    owner: AttemptEvidenceOwner,
    identity: &AttemptLayoutIdentity,
    disposition: AttemptArchiveDisposition,
    archived_at_utc: &str,
) -> Result<(), IoError> {
    let receipt = AttemptArchiveReceiptV1 {
        schema_version: ATTEMPT_ARCHIVE_SCHEMA_V1.to_string(),
        owner,
        identity: identity.clone(),
        disposition,
        archived_at_utc: archived_at_utc.to_string(),
        source_relative_path: owner_relative(
            &paths.source.attempt_dir,
            &paths.owner_root,
            "attempt source",
        )?,
        archive_relative_path: owner_relative(
            &paths.archive_dir,
            &paths.owner_root,
            "attempt archive",
        )?,
        files: inventory(&paths.archive_dir)?,
    };
    publish_or_verify_json(&paths.archive_receipt_path, &receipt)
}

fn validate_receipt_identity(
    receipt: &AttemptArchiveReceiptV1,
    paths: &AttemptArchivePaths,
    owner: AttemptEvidenceOwner,
    identity: &AttemptLayoutIdentity,
    disposition: AttemptArchiveDisposition,
    archived_at_utc: &str,
) -> Result<(), IoError> {
    let expected_source =
        owner_relative(&paths.source.attempt_dir, &paths.owner_root, "attempt source")?;
    let expected_archive =
        owner_relative(&paths.archive_dir, &paths.owner_root, "attempt archive")?;
    if receipt.schema_version != ATTEMPT_ARCHIVE_SCHEMA_V1
        || receipt.owner != owner
        || receipt.identity != *identity
        || receipt.disposition != disposition
        || receipt.archived_at_utc != archived_at_utc
        || receipt.source_relative_path != expected_source
        || receipt.archive_relative_path != expected_archive
        || receipt.files.is_empty()
    {
        return Err(corruption(
            "attempt archive receipt differs from its requested owner or identity",
        ));
    }
    Ok(())
}

fn read_archive_receipt(path: &Path) -> Result<AttemptArchiveReceiptV1, IoError> {
    require_regular_file(path, "attempt archive receipt")?;
    let bytes = read_to_end_bounded(path, MAX_ATTEMPT_ARCHIVE_RECEIPT_BYTES)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| corruption(format!("parse attempt archive receipt: {error}")))
}

fn verify_rollback_receipt(
    paths: &AttemptArchivePaths,
    owner: AttemptEvidenceOwner,
    identity: &AttemptLayoutIdentity,
    disposition: AttemptArchiveDisposition,
    archived_at_utc: &str,
    rolled_back_at_utc: &str,
) -> Result<AttemptArchiveRollbackReceiptV1, IoError> {
    require_regular_file(&paths.rollback_receipt_path, "attempt rollback receipt")?;
    let bytes =
        read_to_end_bounded(&paths.rollback_receipt_path, MAX_ATTEMPT_ARCHIVE_RECEIPT_BYTES)?;
    let receipt: AttemptArchiveRollbackReceiptV1 = serde_json::from_slice(&bytes)
        .map_err(|error| corruption(format!("parse attempt rollback receipt: {error}")))?;
    if receipt.schema_version != ATTEMPT_ARCHIVE_ROLLBACK_SCHEMA_V1
        || receipt.owner != owner
        || receipt.identity != *identity
        || receipt.disposition != disposition
        || receipt.archived_at_utc != archived_at_utc
        || receipt.rolled_back_at_utc != rolled_back_at_utc
        || receipt.archive_receipt_sha256.len() != 64
        || !receipt.archive_receipt_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        || receipt.files.is_empty()
    {
        return Err(corruption("attempt rollback receipt identity differs"));
    }
    Ok(receipt)
}

fn inventory(root: &Path) -> Result<Vec<AttemptArchiveFileV1>, IoError> {
    let mut files = Vec::new();
    collect_inventory(root, root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn collect_inventory(
    root: &Path,
    directory: &Path,
    files: &mut Vec<AttemptArchiveFileV1>,
) -> Result<(), IoError> {
    let mut entries = fs::read_dir(directory)
        .map_err(IoError::from_io)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(IoError::from_io)?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(IoError::from_io)?;
        if metadata.file_type().is_symlink() {
            return Err(corruption(format!(
                "attempt archive contains a symlink: {}",
                path.display()
            )));
        }
        if metadata.is_dir() {
            collect_inventory(root, &path, files)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| corruption("attempt archive inventory escaped its root"))?;
            let relative = normalized_relative(relative)?;
            if relative
                == format!(
                    "{}/{}",
                    super::ATTEMPT_LAYOUT_CONTRACT.manifests_dir,
                    ATTEMPT_ARCHIVE_RECEIPT
                )
            {
                continue;
            }
            files.push(AttemptArchiveFileV1 {
                path: relative,
                size_bytes: metadata.len(),
                sha256: hash_file_sha256(&path)?,
            });
        } else {
            return Err(corruption(format!(
                "attempt archive contains an unsupported filesystem entry: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn verify_inventory(root: &Path, expected: &[AttemptArchiveFileV1]) -> Result<(), IoError> {
    let actual = inventory(root)?;
    if actual != expected {
        return Err(corruption("attempt archive file inventory or digest differs"));
    }
    Ok(())
}

fn owner_relative(path: &Path, owner_root: &Path, label: &str) -> Result<String, IoError> {
    let relative = path
        .strip_prefix(owner_root)
        .map_err(|_| corruption(format!("{label} escaped its owner root")))?;
    normalized_relative(relative)
}

fn normalized_relative(path: &Path) -> Result<String, IoError> {
    if path.as_os_str().is_empty()
        || !path.components().all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(corruption("attempt archive path is not a safe relative path"));
    }
    path.to_str()
        .map(|value| value.replace('\\', "/"))
        .ok_or_else(|| corruption("attempt archive path is not valid UTF-8"))
}

fn publish_or_verify_json<T>(path: &Path, value: &T) -> Result<(), IoError>
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
{
    if path_exists(path)? {
        require_regular_file(path, "immutable attempt transaction receipt")?;
        let bytes = read_to_end_bounded(path, MAX_ATTEMPT_ARCHIVE_RECEIPT_BYTES)?;
        let actual: T = serde_json::from_slice(&bytes)
            .map_err(|error| corruption(format!("parse attempt transaction receipt: {error}")))?;
        if actual != *value {
            return Err(corruption("immutable attempt transaction receipt conflicts"));
        }
        return Ok(());
    }
    atomic_create_json(path, value)
}

fn require_regular_file(path: &Path, label: &str) -> Result<(), IoError> {
    let metadata = fs::symlink_metadata(path).map_err(IoError::from_io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(corruption(format!("{label} is not an owned regular file")));
    }
    Ok(())
}

fn path_exists(path: &Path) -> Result<bool, IoError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(IoError::from_io(error)),
    }
}

fn validate_timestamp(value: &str, label: &str) -> Result<(), IoError> {
    let bytes = value.as_bytes();
    let punctuation = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':'), (19, b'Z')];
    let digits = [0..4, 5..7, 8..10, 11..13, 14..16, 17..19]
        .into_iter()
        .flatten()
        .all(|index| bytes.get(index).is_some_and(u8::is_ascii_digit));
    if bytes.len() != 20
        || !digits
        || punctuation.iter().any(|(index, expected)| bytes.get(*index) != Some(expected))
    {
        return Err(corruption(format!("{label} is not a UTC timestamp")));
    }
    Ok(())
}

fn archive_transaction_schema(schema_version: &str, rollback: bool) -> serde_json::Value {
    let identity = serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["run_directory", "tool_id", "campaign_id", "attempt_id"],
        "properties": {
            "run_directory": {"type": "string", "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"},
            "tool_id": {"type": "string", "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"},
            "campaign_id": {"type": "string", "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"},
            "attempt_id": {"type": "string", "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"}
        }
    });
    if rollback {
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Bijux governed attempt archive rollback receipt",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "schema_version", "owner", "identity", "disposition", "archived_at_utc",
                "rolled_back_at_utc", "archive_receipt_sha256", "files"
            ],
            "properties": {
                "schema_version": {"const": schema_version},
                "owner": {"enum": ["benchmarks", "containers"]},
                "identity": identity,
                "disposition": {"enum": ["failed", "superseded"]},
                "archived_at_utc": {"type": "string", "format": "date-time"},
                "rolled_back_at_utc": {"type": "string", "format": "date-time"},
                "archive_receipt_sha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                "files": file_inventory_schema()
            }
        })
    } else {
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Bijux governed attempt archive receipt",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "schema_version", "owner", "identity", "disposition", "archived_at_utc",
                "source_relative_path", "archive_relative_path", "files"
            ],
            "properties": {
                "schema_version": {"const": schema_version},
                "owner": {"enum": ["benchmarks", "containers"]},
                "identity": identity,
                "disposition": {"enum": ["failed", "superseded"]},
                "archived_at_utc": {"type": "string", "format": "date-time"},
                "source_relative_path": {"type": "string", "minLength": 1},
                "archive_relative_path": {"type": "string", "minLength": 1},
                "files": file_inventory_schema()
            }
        })
    }
}

fn file_inventory_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "minItems": 1,
        "items": {
            "type": "object",
            "additionalProperties": false,
            "required": ["path", "size_bytes", "sha256"],
            "properties": {
                "path": {"type": "string", "minLength": 1},
                "size_bytes": {"type": "integer", "minimum": 0},
                "sha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"}
            }
        }
    })
}

fn corruption(message: impl Into<String>) -> IoError {
    IoError::new(IoErrorKind::Corruption, message)
}

#[cfg(test)]
mod serialization_tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn archive_paths_are_operator_serializable() -> anyhow::Result<()> {
        let paths = attempt_archive_paths(
            Path::new("/study/containers"),
            &AttemptLayoutIdentity {
                run_directory: "run-1001-tool".to_string(),
                tool_id: "tool".to_string(),
                campaign_id: "qualification".to_string(),
                attempt_id: "attempt-001".to_string(),
            },
        )?;
        let encoded = serde_json::to_value(paths)?;

        assert_eq!(
            encoded["source"]["attempt_dir"],
            "/study/containers/run-1001-tool/tool/attempts/qualification/attempt-001"
        );
        assert_eq!(
            encoded["archive_dir"],
            "/study/containers/.attempt-archives/run-1001-tool/tool/qualification/attempt-001"
        );
        Ok(())
    }
}
