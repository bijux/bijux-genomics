//! Immutable, owner-scoped attempt sequence reservation.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::DateTime;
use serde::{Deserialize, Serialize};

pub const ATTEMPT_SEQUENCE_RESERVATION_SCHEMA_V1: &str =
    "bijux.runtime.attempt_sequence_reservation.v1";
const RESERVATION_FILE: &str = "attempt-reservation.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptSequenceIdentity {
    pub run_directory: String,
    pub tool_id: String,
    pub campaign_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptSequenceReservation {
    pub schema_version: String,
    pub owner: bijux_dna_infra::AttemptEvidenceOwner,
    pub identity: bijux_dna_infra::AttemptLayoutIdentity,
    pub reservation_key_sha256: String,
    pub reserved_at_utc: String,
}

/// Reserve or recover the next immutable attempt identity for one owner-scoped series.
///
/// A repeated call with the same reservation key returns the retained active reservation. The
/// sequence considers active and archived attempts, so a failed attempt ID is never reused.
/// Reservation and allocation run under a non-waiting series lock.
///
/// # Errors
/// Returns an error for an invalid identity, timestamp, digest, ambiguous retained reservation,
/// malformed attempt directory, lock contention, or an I/O failure.
pub fn reserve_attempt_sequence(
    owner_root: &Path,
    owner: bijux_dna_infra::AttemptEvidenceOwner,
    sequence: &AttemptSequenceIdentity,
    reservation_key_sha256: &str,
    reserved_at_utc: &str,
) -> Result<AttemptSequenceReservation> {
    validate_request(sequence, reservation_key_sha256, reserved_at_utc)?;
    let lock_path = owner_root
        .join(".attempt-sequence-locks")
        .join(&sequence.run_directory)
        .join(&sequence.tool_id)
        .join(format!("{}.lock", sequence.campaign_id));
    let _lock = bijux_dna_infra::FileLock::try_acquire(&lock_path)
        .context("acquire attempt sequence reservation lock")?;

    let active = series_root(owner_root, sequence, false);
    let archived = series_root(owner_root, sequence, true);
    let mut maximum = 0_u64;
    let mut active_attempt_count = 0_usize;
    let mut retained = None;
    for root in [&active, &archived] {
        for (attempt_id, sequence_number) in attempt_directories(root)? {
            maximum = maximum.max(sequence_number);
            if root == &active {
                active_attempt_count += 1;
            }
            let reservation_path = root.join(&attempt_id).join("manifests").join(RESERVATION_FILE);
            if regular_file_exists(&reservation_path)? {
                let reservation: AttemptSequenceReservation = serde_json::from_slice(
                    &bijux_dna_infra::read_to_end_bounded(&reservation_path, 1024 * 1024)?,
                )
                .with_context(|| format!("parse {}", reservation_path.display()))?;
                validate_reservation(&reservation, owner, sequence)?;
                if root == &active
                    && reservation.reservation_key_sha256 == reservation_key_sha256
                    && retained.replace(reservation).is_some()
                {
                    bail!("attempt sequence contains duplicate active reservations");
                }
            }
        }
    }
    if let Some(reservation) = retained {
        if active_attempt_count != 1 {
            bail!("attempt sequence contains an ambiguous active reservation");
        }
        return Ok(reservation);
    }
    if active_attempt_count != 0 {
        bail!("attempt sequence already has an active attempt for another reservation key");
    }

    let next = maximum.checked_add(1).context("attempt sequence exhausted")?;
    let attempt_id = format!("attempt-{next:03}");
    let identity = bijux_dna_infra::AttemptLayoutIdentity {
        run_directory: sequence.run_directory.clone(),
        tool_id: sequence.tool_id.clone(),
        campaign_id: sequence.campaign_id.clone(),
        execution_unit_id: None,
        attempt_id,
    };
    let layout = bijux_dna_infra::ensure_attempt_layout(owner_root, owner, &identity)
        .context("reserve immutable attempt layout")?;
    let reservation = AttemptSequenceReservation {
        schema_version: ATTEMPT_SEQUENCE_RESERVATION_SCHEMA_V1.to_string(),
        owner,
        identity,
        reservation_key_sha256: reservation_key_sha256.to_string(),
        reserved_at_utc: reserved_at_utc.to_string(),
    };
    bijux_dna_infra::atomic_create_json(&layout.manifests_dir.join(RESERVATION_FILE), &reservation)
        .context("publish attempt sequence reservation")?;
    Ok(reservation)
}

/// Archive an active reserved attempt as superseded without reusing its sequence number.
///
/// # Errors
/// Returns an error when a retained reservation conflicts with the requested owner/identity or
/// when the owner-scoped archive transaction cannot be completed. Owner-scoped attempts created
/// before reservation receipts existed remain supersedable through their layout identity.
pub fn supersede_attempt_sequence(
    owner_root: &Path,
    owner: bijux_dna_infra::AttemptEvidenceOwner,
    identity: &bijux_dna_infra::AttemptLayoutIdentity,
    superseded_at_utc: &str,
) -> Result<bijux_dna_infra::AttemptArchivePaths> {
    let sequence = AttemptSequenceIdentity {
        run_directory: identity.run_directory.clone(),
        tool_id: identity.tool_id.clone(),
        campaign_id: identity.campaign_id.clone(),
    };
    let reservation_path = series_root(owner_root, &sequence, false)
        .join(&identity.attempt_id)
        .join("manifests")
        .join(RESERVATION_FILE);
    if regular_file_exists(&reservation_path)? {
        let reservation: AttemptSequenceReservation = serde_json::from_slice(
            &bijux_dna_infra::read_to_end_bounded(&reservation_path, 1024 * 1024)?,
        )
        .with_context(|| format!("parse {}", reservation_path.display()))?;
        validate_reservation(&reservation, owner, &sequence)?;
        if reservation.identity != *identity {
            bail!("superseded attempt identity conflicts with its reservation");
        }
    }
    bijux_dna_infra::archive_attempt_layout(
        owner_root,
        owner,
        identity,
        bijux_dna_infra::AttemptArchiveDisposition::Superseded,
        superseded_at_utc,
    )
    .context("archive superseded attempt sequence")
}

fn series_root(owner_root: &Path, sequence: &AttemptSequenceIdentity, archived: bool) -> PathBuf {
    if archived {
        owner_root
            .join(".attempt-archives")
            .join(&sequence.run_directory)
            .join(&sequence.tool_id)
            .join(&sequence.campaign_id)
    } else {
        owner_root
            .join(&sequence.run_directory)
            .join(&sequence.tool_id)
            .join("attempts")
            .join(&sequence.campaign_id)
    }
}

fn attempt_directories(root: &Path) -> Result<Vec<(String, u64)>> {
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            bail!("attempt sequence root is not a regular directory: {}", root.display())
        }
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).with_context(|| format!("inspect {}", root.display())),
    }
    let mut attempts = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() || !file_type.is_dir() {
            bail!("attempt sequence contains a non-directory entry");
        }
        let attempt_id = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("attempt sequence contains a non-Unicode identity"))?;
        let sequence = parse_attempt_id(&attempt_id)?;
        attempts.push((attempt_id, sequence));
    }
    Ok(attempts)
}

fn parse_attempt_id(value: &str) -> Result<u64> {
    let number =
        value.strip_prefix("attempt-").context("attempt identity does not use attempt-NNN form")?;
    if number.len() < 3 || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("attempt identity does not use attempt-NNN form");
    }
    let parsed = number.parse::<u64>().context("parse attempt sequence")?;
    if parsed == 0 {
        bail!("attempt sequence must be nonzero");
    }
    Ok(parsed)
}

fn validate_request(
    sequence: &AttemptSequenceIdentity,
    reservation_key_sha256: &str,
    reserved_at_utc: &str,
) -> Result<()> {
    for (value, label) in [
        (&sequence.run_directory, "run directory"),
        (&sequence.tool_id, "tool"),
        (&sequence.campaign_id, "campaign"),
    ] {
        if value.is_empty()
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            bail!("attempt sequence {label} identity is invalid");
        }
    }
    if reservation_key_sha256.len() != 64
        || !reservation_key_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("attempt sequence reservation key is not a lowercase SHA-256 digest");
    }
    let timestamp = DateTime::parse_from_rfc3339(reserved_at_utc)
        .context("parse attempt reservation timestamp")?;
    if timestamp.offset().local_minus_utc() != 0 || !reserved_at_utc.ends_with('Z') {
        bail!("attempt reservation timestamp must use UTC Z notation");
    }
    Ok(())
}

fn validate_reservation(
    reservation: &AttemptSequenceReservation,
    owner: bijux_dna_infra::AttemptEvidenceOwner,
    sequence: &AttemptSequenceIdentity,
) -> Result<()> {
    if reservation.schema_version != ATTEMPT_SEQUENCE_RESERVATION_SCHEMA_V1
        || reservation.owner != owner
        || reservation.identity.run_directory != sequence.run_directory
        || reservation.identity.tool_id != sequence.tool_id
        || reservation.identity.campaign_id != sequence.campaign_id
    {
        bail!("attempt sequence reservation identity is invalid");
    }
    parse_attempt_id(&reservation.identity.attempt_id)?;
    validate_request(sequence, &reservation.reservation_key_sha256, &reservation.reserved_at_utc)
}

fn regular_file_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            bail!("attempt sequence reservation is not a regular file")
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sequence() -> AttemptSequenceIdentity {
        AttemptSequenceIdentity {
            run_directory: "run-1150".to_string(),
            tool_id: "rcorrector".to_string(),
            campaign_id: "qualification".to_string(),
        }
    }

    #[test]
    fn reservations_are_idempotent_and_never_reuse_archived_sequences() -> Result<()> {
        let root = bijux_dna_infra::temp_dir("attempt-sequence")?;
        let owner = bijux_dna_infra::AttemptEvidenceOwner::Containers;
        let first = reserve_attempt_sequence(
            root.path(),
            owner,
            &sequence(),
            &"1".repeat(64),
            "2026-08-13T20:00:00Z",
        )?;
        let replay = reserve_attempt_sequence(
            root.path(),
            owner,
            &sequence(),
            &"1".repeat(64),
            "2026-08-13T20:01:00Z",
        )?;
        assert_eq!(first, replay);
        assert_eq!(first.identity.attempt_id, "attempt-001");
        assert!(reserve_attempt_sequence(
            root.path(),
            owner,
            &sequence(),
            &"2".repeat(64),
            "2026-08-13T20:01:00Z",
        )
        .is_err());

        bijux_dna_infra::archive_attempt_layout(
            root.path(),
            owner,
            &first.identity,
            bijux_dna_infra::AttemptArchiveDisposition::Failed,
            "2026-08-13T20:02:00Z",
        )?;
        let second = reserve_attempt_sequence(
            root.path(),
            owner,
            &sequence(),
            &"2".repeat(64),
            "2026-08-13T20:03:00Z",
        )?;
        assert_eq!(second.identity.attempt_id, "attempt-002");
        Ok(())
    }

    #[test]
    fn reservations_reject_invalid_identity_and_timestamp() {
        let root = bijux_dna_infra::temp_dir("attempt-sequence-invalid").expect("temporary root");
        let owner = bijux_dna_infra::AttemptEvidenceOwner::Containers;
        let mut invalid = sequence();
        invalid.tool_id = "../tool".to_string();
        assert!(reserve_attempt_sequence(
            root.path(),
            owner,
            &invalid,
            &"1".repeat(64),
            "2026-08-13T20:00:00Z",
        )
        .is_err());
        assert!(reserve_attempt_sequence(
            root.path(),
            owner,
            &sequence(),
            &"1".repeat(64),
            "2026-08-13T22:00:00+02:00",
        )
        .is_err());
    }

    #[test]
    fn supersession_archives_the_exact_reserved_identity() -> Result<()> {
        let root = bijux_dna_infra::temp_dir("attempt-sequence-superseded")?;
        let owner = bijux_dna_infra::AttemptEvidenceOwner::Containers;
        let reservation = reserve_attempt_sequence(
            root.path(),
            owner,
            &sequence(),
            &"3".repeat(64),
            "2026-08-13T20:00:00Z",
        )?;

        let archived = supersede_attempt_sequence(
            root.path(),
            owner,
            &reservation.identity,
            "2026-08-13T20:01:00Z",
        )?;

        assert!(!archived.source.attempt_dir.exists());
        assert!(archived.archive_dir.is_dir());
        assert!(supersede_attempt_sequence(
            root.path(),
            owner,
            &reservation.identity,
            "2026-08-13T20:02:00Z",
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn supersession_preserves_owner_scoped_attempts_from_before_reservations() -> Result<()> {
        let root = bijux_dna_infra::temp_dir("attempt-sequence-legacy-superseded")?;
        let owner = bijux_dna_infra::AttemptEvidenceOwner::Containers;
        let identity = bijux_dna_infra::AttemptLayoutIdentity {
            run_directory: "run-1150".to_string(),
            tool_id: "rcorrector".to_string(),
            campaign_id: "qualification".to_string(),
            execution_unit_id: None,
            attempt_id: "attempt-004".to_string(),
        };
        bijux_dna_infra::ensure_attempt_layout(root.path(), owner, &identity)?;

        let archived =
            supersede_attempt_sequence(root.path(), owner, &identity, "2026-08-13T20:01:00Z")?;

        assert!(archived.archive_dir.is_dir());
        Ok(())
    }
}
