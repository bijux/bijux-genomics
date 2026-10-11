//! Validation for benchmark semantic replay receipts.

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkReplayOutcome, BenchmarkReplayReceipt, BenchmarkReplaySemanticIdentity,
};

use super::publication::{
    validate_attempt_selection, validate_commit, validate_component, validate_digest,
    validate_evidence_reference, validate_evidence_set, validate_limitations,
    validate_utc_timestamp,
};
use super::{benchmark_run_directory_name, BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1};

/// Validate a semantic replay receipt without requiring identical scheduler metadata.
///
/// # Errors
/// Returns an error when source locks, attempt selection, semantic digests, or replay outcome are
/// incomplete or contradictory.
pub fn validate_benchmark_replay_receipt(
    receipt: &BenchmarkReplayReceipt,
) -> Result<(), BenchError> {
    if receipt.schema_version != BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1 {
        return Err(invalid("benchmark replay-receipt schema mismatch"));
    }
    validate_component(&receipt.replay_id, "replay_id")?;
    validate_utc_timestamp(&receipt.replayed_at_utc, "replayed_at_utc")?;
    benchmark_run_directory_name(&receipt.run)?;
    validate_component(&receipt.publication_id, "publication_id")?;
    validate_attempt_selection(&receipt.run, &receipt.selected_attempts, "replay")?;
    validate_commit(&receipt.source_repository_commit)?;
    validate_commit(&receipt.replay_repository_commit)?;
    validate_evidence_reference(&receipt.source_publication)?;
    validate_evidence_reference(&receipt.run_definition)?;
    if receipt.source_publication.uri == receipt.run_definition.uri {
        return Err(invalid(
            "benchmark replay source publication and run definition reuse one URI",
        ));
    }
    if receipt.case_definitions.len() != receipt.selected_attempts.len() {
        return Err(invalid(
            "benchmark replay case-definition closure differs from selected attempts",
        ));
    }
    validate_evidence_set(&receipt.case_definitions)?;
    validate_evidence_set(&receipt.immutable_inputs)?;
    validate_semantic_identity(&receipt.expected)?;
    if let Some(observed) = &receipt.observed {
        validate_semantic_identity(observed)?;
    }
    validate_replay_outcome(receipt)?;
    validate_limitations(&receipt.limitations)?;
    Ok(())
}

fn validate_semantic_identity(
    identity: &BenchmarkReplaySemanticIdentity,
) -> Result<(), BenchError> {
    validate_digest(&identity.decision_sha256)?;
    validate_digest(&identity.report_sha256)?;
    validate_digest(&identity.evidence_book_sha256)?;
    Ok(())
}

fn validate_replay_outcome(receipt: &BenchmarkReplayReceipt) -> Result<(), BenchError> {
    match (receipt.outcome, receipt.observed.as_ref()) {
        (BenchmarkReplayOutcome::ExactMatch, Some(observed))
            if observed == &receipt.expected && receipt.differences.is_empty() => {}
        (BenchmarkReplayOutcome::Mismatch, Some(observed))
            if observed != &receipt.expected
                && has_substantive_differences(&receipt.differences) => {}
        (BenchmarkReplayOutcome::Blocked, None)
            if has_substantive_differences(&receipt.differences) => {}
        (BenchmarkReplayOutcome::ExactMatch, _) => {
            return Err(invalid(
                "exact benchmark replay requires identical observed semantics and no differences",
            ));
        }
        (BenchmarkReplayOutcome::Mismatch, _) => {
            return Err(invalid(
                "mismatched benchmark replay requires divergent observed semantics and differences",
            ));
        }
        (BenchmarkReplayOutcome::Blocked, _) => {
            return Err(invalid(
                "blocked benchmark replay requires no observed semantics and an actionable reason",
            ));
        }
    }
    Ok(())
}

fn has_substantive_differences(differences: &[String]) -> bool {
    !differences.is_empty()
        && differences.iter().all(|value| value.trim().len() >= 10 && value == value.trim())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
