//! Validation for the paper-facing benchmark run index.

use std::collections::BTreeSet;

use crate::model::{BenchmarkRunIndex, BenchmarkRunRegistryState};
use crate::BenchError;

use super::{validate_attempt_identity, BENCHMARK_RUN_INDEX_SCHEMA_V1};

/// Validate run ordering, protocol identities, publication closure, and summary counts.
///
/// # Errors
/// Returns [`BenchError`] when the index is incomplete, contradictory, or not citation-safe.
pub fn validate_benchmark_run_index(index: &BenchmarkRunIndex) -> Result<(), BenchError> {
    if index.schema_version != BENCHMARK_RUN_INDEX_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy("unsupported benchmark run index schema".into()));
    }
    validate_reference(
        &index.source_registry.uri,
        &index.source_registry.sha256,
        index.source_registry.size_bytes,
    )?;
    if index.source_registry.uri != "repo://benchmarks/contracts/run-registry.json" {
        return Err(BenchError::InvalidPolicy(
            "benchmark run index source registry drifted".into(),
        ));
    }

    let mut run_ids = BTreeSet::new();
    let mut citation_keys = BTreeSet::new();
    let mut previous_run_id = None;
    let mut active = 0;
    let mut reserved = 0;
    let mut retired = 0;
    let mut citable = 0;
    for run in &index.runs {
        if !is_run_id(&run.run_id)
            || previous_run_id.is_some_and(|previous| previous >= run.run_id.as_str())
            || !run_ids.insert(run.run_id.as_str())
            || !citation_keys.insert(run.citation_key.as_str())
        {
            return Err(BenchError::InvalidPolicy(
                "benchmark run index identities are invalid, duplicated, or unsorted".into(),
            ));
        }
        previous_run_id = Some(run.run_id.as_str());
        validate_entry(run)?;
        match run.protocol_state {
            BenchmarkRunRegistryState::Active => active += 1,
            BenchmarkRunRegistryState::Reserved => reserved += 1,
            BenchmarkRunRegistryState::Retired => retired += 1,
        }
        citable += usize::from(run.selected_publication.is_some());
    }
    let summary = &index.summary;
    if summary.registered_run_count != index.runs.len()
        || summary.active_protocol_count != active
        || summary.reserved_protocol_count != reserved
        || summary.retired_protocol_count != retired
        || summary.citable_publication_count != citable
        || summary.unpublished_run_count != index.runs.len() - citable
    {
        return Err(BenchError::InvalidPolicy("benchmark run index summary drifted".into()));
    }
    Ok(())
}

fn validate_entry(run: &crate::model::BenchmarkRunIndexEntry) -> Result<(), BenchError> {
    if run.citation_key != format!("bijux-benchmark-{}", run.run_id)
        || run.protocol_title.trim().is_empty()
        || run.slug.trim().is_empty()
        || run.stage_id.trim().is_empty()
        || run.owner != "bijux-dna-bench"
        || run.registry_uri
            != format!("repo://benchmarks/contracts/run-registry.json#{}", run.run_id)
        || run.tool_ids.is_empty()
        || run.corpus_roles.is_empty()
        || run.acceptance_policy_id.trim().is_empty()
        || !valid_sha256(&run.protocol_sha256)
    {
        return Err(BenchError::InvalidPolicy(format!(
            "benchmark run index entry is incomplete: {}",
            run.run_id
        )));
    }
    validate_sorted_unique(&run.tool_ids, "run index tools")?;
    validate_sorted_unique(&run.corpus_roles, "run index corpus roles")?;
    match &run.selected_publication {
        Some(publication) => {
            if run.protocol_state != BenchmarkRunRegistryState::Active
                || !run.limitations.is_empty()
                || publication.publication_id.trim().is_empty()
                || publication.selected_attempts.is_empty()
                || publication.repository_commit.len() != 40
                || !publication.repository_commit.bytes().all(|byte| byte.is_ascii_hexdigit())
                || publication.access_statement.trim().is_empty()
            {
                return Err(BenchError::InvalidPolicy(format!(
                    "benchmark run index publication is incomplete: {}",
                    run.run_id
                )));
            }
            for attempt in &publication.selected_attempts {
                validate_attempt_identity(attempt)?;
                if attempt.run.run_id != run.run_id || attempt.run.stage_id != run.stage_id {
                    return Err(BenchError::InvalidPolicy(
                        "run index selected attempt targets another protocol".into(),
                    ));
                }
            }
            for reference in [
                &publication.report,
                &publication.evidence_book,
                &publication.publication_receipt,
                &publication.citation_record,
                &publication.replay_receipt,
            ] {
                validate_reference(&reference.uri, &reference.sha256, reference.size_bytes)?;
            }
        }
        None if run.limitations.is_empty() => {
            return Err(BenchError::InvalidPolicy(format!(
                "unpublished benchmark run lacks an explicit limitation: {}",
                run.run_id
            )));
        }
        None => {}
    }
    Ok(())
}

fn validate_reference(uri: &str, sha256: &str, size_bytes: u64) -> Result<(), BenchError> {
    if !(uri.starts_with("repo://") || uri.starts_with("study://"))
        || !valid_sha256(sha256)
        || size_bytes == 0
    {
        return Err(BenchError::InvalidPolicy("invalid benchmark run index reference".into()));
    }
    Ok(())
}

fn validate_sorted_unique(values: &[String], label: &str) -> Result<(), BenchError> {
    if values.iter().any(|value| value.trim().is_empty())
        || values.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(BenchError::InvalidPolicy(format!("invalid {label}")));
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_run_id(value: &str) -> bool {
    value.len() == 8
        && value.starts_with("run-")
        && value[4..].bytes().all(|byte| byte.is_ascii_digit())
}
