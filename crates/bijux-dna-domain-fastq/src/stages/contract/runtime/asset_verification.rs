use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::artifacts::{
    AssetVerificationEntryV1, AssetVerificationStatusV1, VerifyAssetsReportV1,
    VERIFY_ASSETS_REPORT_SCHEMA_VERSION,
};

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetLockRecord {
    path: String,
    sha256: String,
}

/// Verify locked assets prior to execution.
///
/// # Errors
/// Returns an error when lock files cannot be read or parsed.
pub fn verify_assets(asset_locks: &[&Path]) -> Result<VerifyAssetsReportV1> {
    if asset_locks.is_empty() {
        return Err(anyhow!("fastq.verify_assets requires at least one asset lock"));
    }

    let mut entries = Vec::with_capacity(asset_locks.len());

    for lock_path in asset_locks {
        let raw = std::fs::read_to_string(lock_path)
            .map_err(|err| anyhow!("read {}: {err}", lock_path.display()))?;
        let parsed = serde_json::from_str::<AssetLockRecord>(&raw);

        match parsed {
            Ok(lock) => {
                let asset_path = resolve_asset_path(lock_path, Path::new(&lock.path));
                if !asset_path.exists() {
                    entries.push(AssetVerificationEntryV1 {
                        lock_path: lock_path.display().to_string(),
                        asset_path: Some(asset_path.display().to_string()),
                        expected_sha256: Some(lock.sha256),
                        observed_sha256: None,
                        status: AssetVerificationStatusV1::Missing,
                        reason: Some("declared asset path is missing".to_string()),
                    });
                    continue;
                }

                let observed = bijux_dna_infra::hash_file_sha256(&asset_path)
                    .map_err(|err| anyhow!("hash {}: {err}", asset_path.display()))?;
                if observed != lock.sha256 {
                    entries.push(AssetVerificationEntryV1 {
                        lock_path: lock_path.display().to_string(),
                        asset_path: Some(asset_path.display().to_string()),
                        expected_sha256: Some(lock.sha256),
                        observed_sha256: Some(observed),
                        status: AssetVerificationStatusV1::Mismatch,
                        reason: Some("asset checksum mismatch".to_string()),
                    });
                    continue;
                }

                entries.push(AssetVerificationEntryV1 {
                    lock_path: lock_path.display().to_string(),
                    asset_path: Some(asset_path.display().to_string()),
                    expected_sha256: Some(lock.sha256),
                    observed_sha256: Some(observed),
                    status: AssetVerificationStatusV1::Verified,
                    reason: None,
                });
            }
            Err(err) => {
                entries.push(AssetVerificationEntryV1 {
                    lock_path: lock_path.display().to_string(),
                    asset_path: None,
                    expected_sha256: None,
                    observed_sha256: None,
                    status: AssetVerificationStatusV1::InvalidLock,
                    reason: Some(format!("invalid lock format: {err}")),
                });
            }
        }
    }

    entries.sort_by(|left, right| left.lock_path.cmp(&right.lock_path));

    let verified_asset_count = entries
        .iter()
        .filter(|entry| entry.status == AssetVerificationStatusV1::Verified)
        .count() as u64;
    let missing_asset_count = entries
        .iter()
        .filter(|entry| entry.status == AssetVerificationStatusV1::Missing)
        .count() as u64;
    let mismatched_asset_count = entries
        .iter()
        .filter(|entry| entry.status == AssetVerificationStatusV1::Mismatch)
        .count() as u64;
    let invalid_lock_count = entries
        .iter()
        .filter(|entry| entry.status == AssetVerificationStatusV1::InvalidLock)
        .count() as u64;

    Ok(VerifyAssetsReportV1 {
        schema_version: VERIFY_ASSETS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.verify_assets".to_string(),
        stage_id: "fastq.verify_assets".to_string(),
        tool_id: "bijux".to_string(),
        verified_asset_count,
        missing_asset_count,
        mismatched_asset_count,
        invalid_lock_count,
        entries,
    })
}

/// Refuse execution when asset verification is not clean.
///
/// # Errors
/// Returns an error if any asset is missing, mismatched, or invalid.
pub fn ensure_assets_verified(report: &VerifyAssetsReportV1) -> Result<()> {
    if report.missing_asset_count > 0 {
        return Err(anyhow!(
            "fastq.verify_assets failed: {} assets missing",
            report.missing_asset_count
        ));
    }
    if report.mismatched_asset_count > 0 {
        return Err(anyhow!(
            "fastq.verify_assets failed: {} assets mismatched",
            report.mismatched_asset_count
        ));
    }
    if report.invalid_lock_count > 0 {
        return Err(anyhow!(
            "fastq.verify_assets failed: {} invalid lock records",
            report.invalid_lock_count
        ));
    }
    Ok(())
}

fn resolve_asset_path(lock_path: &Path, declared: &Path) -> PathBuf {
    if declared.is_absolute() {
        return declared.to_path_buf();
    }
    lock_path.parent().unwrap_or_else(|| Path::new(".")).join(declared)
}

#[cfg(test)]
mod tests {
    use super::{ensure_assets_verified, verify_assets};

    #[test]
    fn verify_assets_reports_verified_and_mismatch_states() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-fastq-assets")?;
        let good_asset = temp.path().join("good.fa");
        let bad_asset = temp.path().join("bad.fa");
        std::fs::write(&good_asset, ">a\nACGT\n")?;
        std::fs::write(&bad_asset, ">b\nTGCA\n")?;

        let good_digest = bijux_dna_infra::hash_file_sha256(&good_asset)
            .map_err(|err| anyhow::anyhow!("hash good asset: {err}"))?;
        let good_lock = temp.path().join("good.lock.json");
        let bad_lock = temp.path().join("bad.lock.json");

        std::fs::write(
            &good_lock,
            serde_json::json!({
                "path": "good.fa",
                "sha256": good_digest,
            })
            .to_string(),
        )?;
        std::fs::write(
            &bad_lock,
            serde_json::json!({
                "path": "bad.fa",
                "sha256": "deadbeef",
            })
            .to_string(),
        )?;

        let report = verify_assets(&[good_lock.as_path(), bad_lock.as_path()])?;
        assert_eq!(report.verified_asset_count, 1);
        assert_eq!(report.mismatched_asset_count, 1);
        assert!(ensure_assets_verified(&report).is_err());
        Ok(())
    }
}
