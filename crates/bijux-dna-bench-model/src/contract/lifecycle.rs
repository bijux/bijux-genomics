//! Validation for benchmark attempt lifecycle records.

use std::collections::BTreeSet;

use crate::diagnostics::BenchError;
use crate::model::{BenchmarkAttemptArchive, BenchmarkAttemptDisposition};

use super::BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1;

/// Validate a benchmark-owned terminal attempt archive.
///
/// # Errors
/// Returns an error when identity, disposition, source, or file evidence is incomplete.
pub fn validate_attempt_archive(archive: &BenchmarkAttemptArchive) -> Result<(), BenchError> {
    if archive.schema_version != BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1 {
        return Err(invalid("benchmark attempt archive schema mismatch"));
    }
    for (value, field) in [
        (&archive.run_id, "run_id"),
        (&archive.attempt_id, "attempt_id"),
        (&archive.tool_id, "tool_id"),
        (&archive.slurm_job_id, "slurm_job_id"),
        (&archive.archived_at_utc, "archived_at_utc"),
    ] {
        require_text(value, field)?;
    }
    if !archive.slurm_job_id.bytes().all(|byte| byte.is_ascii_digit())
        || archive.attempt_id != format!("slurm-{}", archive.slurm_job_id)
    {
        return Err(invalid("benchmark attempt archive scheduler identity is invalid"));
    }
    match (&archive.disposition, archive.reason.as_deref()) {
        (BenchmarkAttemptDisposition::Failed, None) => {}
        (BenchmarkAttemptDisposition::Superseded, Some(reason)) if reason.trim().len() >= 20 => {}
        (BenchmarkAttemptDisposition::Failed, Some(_)) => {
            return Err(invalid("failed benchmark attempt must not claim a supersession reason"));
        }
        (BenchmarkAttemptDisposition::Superseded, _) => {
            return Err(invalid("superseded benchmark attempt requires a substantive reason"));
        }
    }
    for uri in [archive.source_paths.submission.as_str(), archive.source_paths.result.as_str()] {
        if !uri.starts_with("study://") || uri.contains("..") {
            return Err(invalid("benchmark attempt archive source URI is invalid"));
        }
    }
    for digest in [&archive.evidence_sha256, &archive.accounting_sha256] {
        validate_digest(digest)?;
    }
    if archive.files.is_empty() {
        return Err(invalid("benchmark attempt archive contains no file inventory"));
    }
    let mut paths = BTreeSet::new();
    for file in &archive.files {
        if !(file.path.starts_with("submission/") || file.path.starts_with("result/"))
            || file.path.contains("..")
            || !paths.insert(file.path.as_str())
        {
            return Err(invalid("benchmark attempt archive file path is invalid or duplicated"));
        }
        validate_digest(&file.sha256)?;
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return Err(invalid(format!("benchmark attempt archive {field} is blank")));
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), BenchError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid("benchmark attempt archive SHA-256 identity is malformed"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
