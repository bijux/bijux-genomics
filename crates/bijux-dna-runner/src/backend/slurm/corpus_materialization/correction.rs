//! Owner: bijux-dna-runner
//! Immutable corrections for operator-supplied materialization evidence timestamps.

use std::path::Path;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};

use super::contracts::{
    CorpusMaterializationEvidence, CorpusMaterializationEvidenceCorrection,
    CorpusMaterializationEvidenceCorrectionRequest,
    CORPUS_MATERIALIZATION_EVIDENCE_CORRECTION_SCHEMA_V1,
};
use super::validation::{ensure_json_identity, read_json, require_utc_timestamp};

/// Publish a digest-bound correction without replacing the original terminal evidence.
///
/// # Errors
/// Returns an error when the original evidence, timestamp conversion, correction identity, or
/// immutable destination is invalid.
pub fn publish_corpus_materialization_evidence_correction(
    request: &CorpusMaterializationEvidenceCorrectionRequest,
) -> Result<CorpusMaterializationEvidenceCorrection> {
    require_utc_timestamp(&request.corrected_at_utc, "correction publication timestamp")?;
    validate_source_commit(&request.source_commit)?;
    validate_text(&request.source_timezone, "source timezone", 3)?;
    if !request.source_timezone.contains('/') {
        bail!("source timezone must be an explicit IANA-style identifier");
    }
    validate_text(&request.reason, "correction reason", 20)?;

    let evidence_path = canonical_regular_file(&request.evidence_path)?;
    if evidence_path.file_name().and_then(|name| name.to_str())
        != Some("materialization-evidence.json")
    {
        bail!("correction source must be the terminal materialization evidence authority");
    }
    let expected_output = evidence_path
        .parent()
        .context("materialization evidence has no attempt directory")?
        .join("materialization-evidence-correction.json");
    if request.output_path != expected_output {
        bail!("materialization evidence correction must be published beside its source evidence");
    }
    let evidence: CorpusMaterializationEvidence = read_json(&evidence_path)?;
    require_utc_timestamp(&evidence.collected_at_utc, "recorded collection timestamp")?;
    let observed_local = DateTime::parse_from_rfc3339(&request.observed_local_time)
        .context("parse observed local collection timestamp")?;
    if observed_local.offset().local_minus_utc() == 0 {
        bail!("observed local collection timestamp must carry the non-UTC source offset");
    }
    let corrected_value =
        observed_local.with_timezone(&Utc).to_rfc3339_opts(SecondsFormat::Secs, true);
    if corrected_value == evidence.collected_at_utc {
        bail!("corrected collection timestamp must differ from the recorded value");
    }
    let corrected_at = DateTime::parse_from_rfc3339(&request.corrected_at_utc)
        .context("parse correction publication timestamp")?;
    if corrected_at < observed_local {
        bail!("correction publication timestamp precedes the corrected collection instant");
    }

    let correction = CorpusMaterializationEvidenceCorrection {
        schema_version: CORPUS_MATERIALIZATION_EVIDENCE_CORRECTION_SCHEMA_V1.to_string(),
        run_id: evidence.run_id,
        attempt_id: evidence.attempt_id,
        job_id: evidence.job_id,
        evidence_path,
        evidence_sha256: bijux_dna_infra::hash_file_sha256(&request.evidence_path)
            .context("hash original materialization evidence")?,
        field: "collected_at_utc".to_string(),
        recorded_value: evidence.collected_at_utc,
        observed_local_time: observed_local.to_rfc3339_opts(SecondsFormat::Secs, false),
        source_timezone: request.source_timezone.clone(),
        corrected_value,
        corrected_at_utc: request.corrected_at_utc.clone(),
        source_commit: request.source_commit.clone(),
        reason: request.reason.clone(),
    };
    ensure_json_identity(&request.output_path, &correction)?;
    Ok(correction)
}

fn canonical_regular_file(path: &Path) -> Result<std::path::PathBuf> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect original materialization evidence {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("original materialization evidence is not a regular non-symlink file");
    }
    path.canonicalize().context("canonicalize original materialization evidence")
}

fn validate_source_commit(value: &str) -> Result<()> {
    if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("correction source commit must be a 40-character hexadecimal Git identity");
    }
    Ok(())
}

fn validate_text(value: &str, label: &str, minimum_length: usize) -> Result<()> {
    if value.trim() != value || value.len() < minimum_length || value.contains(['\n', '\r']) {
        bail!("{label} is not substantive single-line text");
    }
    Ok(())
}
