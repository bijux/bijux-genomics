use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::{
    atomic_create_json, atomic_write_json, ensure_dir, read_to_end_bounded, FileLock, IoError,
    IoErrorKind,
};

use super::{
    AttemptEvidenceOwner, AttemptLayoutDirectoriesV1, AttemptLayoutIdentity,
    AttemptLayoutManifestV1, AttemptLayoutPaths, RunLayoutPaths, ATTEMPT_LAYOUT_CONTRACT,
    ATTEMPT_LAYOUT_SCHEMA_V1, RUN_LAYOUT_CONTRACT,
};

const MAX_ATTEMPT_LAYOUT_MANIFEST_BYTES: usize = 64 * 1024;
/// Attempt the run-level lock once for coordinated publish/write operations.
///
/// # Errors
/// Returns an IO error if the lock is contended or cannot be opened.
pub fn try_lock_run(layout: &RunLayoutPaths) -> Result<FileLock, IoError> {
    ensure_dir(&layout.run_dir)?;
    FileLock::try_acquire(&layout.run_dir.join(RUN_LAYOUT_CONTRACT.lock_file))
}

/// Publish a run by writing an atomic marker into the artifacts directory.
///
/// # Errors
/// Returns an IO error if the marker cannot be written.
pub fn publish_run(layout: &RunLayoutPaths, run_id: &str) -> Result<PathBuf, IoError> {
    ensure_dir(&layout.artifacts_dir)?;
    let marker = layout.artifacts_dir.join(RUN_LAYOUT_CONTRACT.publish_marker);
    let payload = serde_json::json!({
        "schema_version": "bijux.run_publish.v1",
        "run_id": run_id,
    });
    atomic_write_json(&marker, &payload)?;
    Ok(marker)
}

/// Derive the governed directory tree for one owner-scoped execution attempt.
///
/// # Errors
/// Returns an error when an identity is empty, hidden, or not one safe path segment.
pub fn attempt_layout_paths(
    owner_root: &Path,
    identity: &AttemptLayoutIdentity,
) -> Result<AttemptLayoutPaths, IoError> {
    for (label, value) in [
        ("run directory", identity.run_directory.as_str()),
        ("tool id", identity.tool_id.as_str()),
        ("campaign id", identity.campaign_id.as_str()),
        ("attempt id", identity.attempt_id.as_str()),
    ] {
        validate_identity_segment(value, label)?;
    }
    if let Some(execution_unit_id) = &identity.execution_unit_id {
        validate_identity_segment(execution_unit_id, "execution unit id")?;
    }
    validate_owner_root(owner_root, "attempt owner root")?;
    let campaign_dir = owner_root
        .join(&identity.run_directory)
        .join(&identity.tool_id)
        .join(ATTEMPT_LAYOUT_CONTRACT.attempts_dir)
        .join(&identity.campaign_id);
    let attempt_dir = identity.execution_unit_id.as_ref().map_or_else(
        || campaign_dir.join(&identity.attempt_id),
        |execution_unit_id| campaign_dir.join(execution_unit_id).join(&identity.attempt_id),
    );
    let manifests_dir = attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.manifests_dir);
    Ok(AttemptLayoutPaths {
        logs_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.logs_dir),
        outputs_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.outputs_dir),
        accounting_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.accounting_dir),
        validation_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.validation_dir),
        science_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.science_dir),
        receipts_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.receipts_dir),
        layout_manifest_path: manifests_dir.join(ATTEMPT_LAYOUT_CONTRACT.layout_manifest),
        manifests_dir,
        attempt_dir,
    })
}

/// Refuse owner roots that overlap and could mix container and benchmark evidence.
///
/// # Errors
/// Returns an error when either root is unsafe, equal to the other, or nested under the other.
pub fn validate_attempt_owner_roots(
    container_root: &Path,
    benchmark_root: &Path,
) -> Result<(), IoError> {
    validate_owner_root(container_root, "container attempt root")?;
    validate_owner_root(benchmark_root, "benchmark attempt root")?;
    let normalized_container = normalized_owner_root(container_root)?;
    let normalized_benchmark = normalized_owner_root(benchmark_root)?;
    if container_root.is_absolute() != benchmark_root.is_absolute()
        || normalized_container == normalized_benchmark
        || normalized_container.starts_with(&normalized_benchmark)
        || normalized_benchmark.starts_with(&normalized_container)
    {
        return Err(corruption(
            "container and benchmark attempt roots must be separate, consistently rooted, non-nested paths",
        ));
    }
    Ok(())
}

/// Create or recover a complete attempt layout under its immutable owner and identity.
///
/// A new layout is assembled in a sibling staging directory and renamed into place, so readers
/// never observe a partial required directory set. An existing layout is accepted only when its
/// manifest and required directories match the requested identity exactly.
///
/// # Errors
/// Returns an error when identity validation, locking, staging, publication, or recovery fails.
pub fn ensure_attempt_layout(
    owner_root: &Path,
    owner: AttemptEvidenceOwner,
    identity: &AttemptLayoutIdentity,
) -> Result<AttemptLayoutPaths, IoError> {
    let layout = attempt_layout_paths(owner_root, identity)?;
    let parent = layout
        .attempt_dir
        .parent()
        .ok_or_else(|| corruption("attempt layout has no parent directory"))?;
    ensure_dir(parent)?;
    let lock_path = attempt_lock_path(owner_root, identity);
    let _lock = FileLock::try_acquire(&lock_path)?;
    if layout.attempt_dir.exists() {
        validate_attempt_layout(&layout, owner, identity)?;
        return Ok(layout);
    }

    let staging = tempfile::Builder::new()
        .prefix(".attempt-layout-")
        .tempdir_in(parent)
        .map_err(IoError::from_io)?;
    let staged = layout_from_attempt_dir(staging.path());
    for directory in required_directories(&staged) {
        ensure_dir(directory)?;
    }
    atomic_create_json(&staged.layout_manifest_path, &layout_manifest(owner, identity))?;
    fs::rename(staging.path(), &layout.attempt_dir).map_err(IoError::from_io)?;
    validate_attempt_layout(&layout, owner, identity)?;
    Ok(layout)
}

/// Return the strict JSON schema for the immutable attempt-layout manifest.
#[must_use]
pub fn attempt_layout_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://bijux.dev/schemas/attempt-layout.v1.json",
        "title": "Bijux governed attempt layout",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "schema_version", "owner", "run_directory", "tool_id", "campaign_id",
            "attempt_id", "directories"
        ],
        "properties": {
            "schema_version": {"const": ATTEMPT_LAYOUT_SCHEMA_V1},
            "owner": {"enum": ["benchmarks", "containers"]},
            "run_directory": {
                "type": "string", "minLength": 1, "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"
            },
            "tool_id": {
                "type": "string", "minLength": 1, "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"
            },
            "campaign_id": {
                "type": "string", "minLength": 1, "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"
            },
            "execution_unit_id": {
                "type": "string", "minLength": 1, "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"
            },
            "attempt_id": {
                "type": "string", "minLength": 1, "pattern": "^[A-Za-z0-9][A-Za-z0-9._-]*$"
            },
            "directories": {
                "type": "object",
                "additionalProperties": false,
                "required": [
                    "manifests", "logs", "outputs", "accounting", "validation", "science",
                    "receipts"
                ],
                "properties": {
                    "manifests": {"const": ATTEMPT_LAYOUT_CONTRACT.manifests_dir},
                    "logs": {"const": ATTEMPT_LAYOUT_CONTRACT.logs_dir},
                    "outputs": {"const": ATTEMPT_LAYOUT_CONTRACT.outputs_dir},
                    "accounting": {"const": ATTEMPT_LAYOUT_CONTRACT.accounting_dir},
                    "validation": {"const": ATTEMPT_LAYOUT_CONTRACT.validation_dir},
                    "science": {"const": ATTEMPT_LAYOUT_CONTRACT.science_dir},
                    "receipts": {"const": ATTEMPT_LAYOUT_CONTRACT.receipts_dir}
                }
            }
        }
    })
}

pub(super) fn validate_attempt_layout(
    layout: &AttemptLayoutPaths,
    owner: AttemptEvidenceOwner,
    identity: &AttemptLayoutIdentity,
) -> Result<(), IoError> {
    require_owned_directory(&layout.attempt_dir, "attempt root")?;
    for directory in required_directories(layout) {
        require_owned_directory(directory, "attempt evidence directory")?;
    }
    let metadata = fs::symlink_metadata(&layout.layout_manifest_path).map_err(IoError::from_io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(corruption("attempt layout manifest is not an owned regular file"));
    }
    let bytes =
        read_to_end_bounded(&layout.layout_manifest_path, MAX_ATTEMPT_LAYOUT_MANIFEST_BYTES)?;
    let actual: AttemptLayoutManifestV1 = serde_json::from_slice(&bytes)
        .map_err(|error| corruption(format!("parse attempt layout manifest: {error}")))?;
    if actual != layout_manifest(owner, identity) {
        return Err(corruption(
            "attempt layout manifest differs from its requested owner or identity",
        ));
    }
    Ok(())
}

fn layout_manifest(
    owner: AttemptEvidenceOwner,
    identity: &AttemptLayoutIdentity,
) -> AttemptLayoutManifestV1 {
    AttemptLayoutManifestV1 {
        schema_version: ATTEMPT_LAYOUT_SCHEMA_V1.to_string(),
        owner,
        run_directory: identity.run_directory.clone(),
        tool_id: identity.tool_id.clone(),
        campaign_id: identity.campaign_id.clone(),
        execution_unit_id: identity.execution_unit_id.clone(),
        attempt_id: identity.attempt_id.clone(),
        directories: AttemptLayoutDirectoriesV1 {
            manifests: ATTEMPT_LAYOUT_CONTRACT.manifests_dir.to_string(),
            logs: ATTEMPT_LAYOUT_CONTRACT.logs_dir.to_string(),
            outputs: ATTEMPT_LAYOUT_CONTRACT.outputs_dir.to_string(),
            accounting: ATTEMPT_LAYOUT_CONTRACT.accounting_dir.to_string(),
            validation: ATTEMPT_LAYOUT_CONTRACT.validation_dir.to_string(),
            science: ATTEMPT_LAYOUT_CONTRACT.science_dir.to_string(),
            receipts: ATTEMPT_LAYOUT_CONTRACT.receipts_dir.to_string(),
        },
    }
}

pub(super) fn layout_from_attempt_dir(attempt_dir: &Path) -> AttemptLayoutPaths {
    let manifests_dir = attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.manifests_dir);
    AttemptLayoutPaths {
        logs_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.logs_dir),
        outputs_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.outputs_dir),
        accounting_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.accounting_dir),
        validation_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.validation_dir),
        science_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.science_dir),
        receipts_dir: attempt_dir.join(ATTEMPT_LAYOUT_CONTRACT.receipts_dir),
        layout_manifest_path: manifests_dir.join(ATTEMPT_LAYOUT_CONTRACT.layout_manifest),
        manifests_dir,
        attempt_dir: attempt_dir.to_path_buf(),
    }
}

pub(super) fn attempt_lock_path(owner_root: &Path, identity: &AttemptLayoutIdentity) -> PathBuf {
    let campaign_lock_dir = owner_root
        .join(".attempt-layout-locks")
        .join(&identity.run_directory)
        .join(&identity.tool_id)
        .join(&identity.campaign_id);
    identity.execution_unit_id.as_ref().map_or_else(
        || campaign_lock_dir.join(format!("{}.lock", identity.attempt_id)),
        |execution_unit_id| {
            campaign_lock_dir.join(execution_unit_id).join(format!("{}.lock", identity.attempt_id))
        },
    )
}

fn required_directories(layout: &AttemptLayoutPaths) -> [&Path; 7] {
    [
        &layout.manifests_dir,
        &layout.logs_dir,
        &layout.outputs_dir,
        &layout.accounting_dir,
        &layout.validation_dir,
        &layout.science_dir,
        &layout.receipts_dir,
    ]
}

fn require_owned_directory(path: &Path, label: &str) -> Result<(), IoError> {
    let metadata = fs::symlink_metadata(path).map_err(IoError::from_io)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(corruption(format!("{label} is not an owned directory")));
    }
    Ok(())
}

fn validate_identity_segment(value: &str, label: &str) -> Result<(), IoError> {
    let mut components = Path::new(value).components();
    let one_normal_component =
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
    let allowed = !value.is_empty()
        && !value.starts_with('.')
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        });
    if !one_normal_component || !allowed {
        return Err(corruption(format!("{label} is not a safe identity segment")));
    }
    Ok(())
}

fn validate_owner_root(path: &Path, label: &str) -> Result<(), IoError> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(corruption(format!("{label} is not a stable path")));
    }
    Ok(())
}

fn normalized_owner_root(path: &Path) -> Result<PathBuf, IoError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(IoError::from_io)?.join(path)
    };
    let mut existing = absolute.as_path();
    loop {
        match fs::symlink_metadata(existing) {
            Ok(_) => {
                let resolved = existing.canonicalize().map_err(IoError::from_io)?;
                let missing = absolute
                    .strip_prefix(existing)
                    .map_err(|_| corruption("attempt owner root normalization escaped its path"))?;
                return Ok(resolved.join(missing));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                existing = existing.parent().ok_or_else(|| {
                    corruption("attempt owner root has no existing path ancestor")
                })?;
            }
            Err(error) => return Err(IoError::from_io(error)),
        }
    }
}

fn corruption(message: impl Into<String>) -> IoError {
    IoError::new(IoErrorKind::Corruption, message)
}
