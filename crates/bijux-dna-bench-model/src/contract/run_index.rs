//! Validation for the paper-facing benchmark run index.

use std::collections::BTreeSet;

use crate::model::{
    BenchmarkProtocolCitation, BenchmarkProtocolCitationStatus, BenchmarkRunCaseIndex,
    BenchmarkRunIndex, BenchmarkRunPublicationPointer, BenchmarkRunRegistryState,
};
use crate::BenchError;

use super::{
    benchmark_run_directory_name, validate_attempt_identity, BENCHMARK_PROTOCOL_CITATION_SCHEMA_V1,
    BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1, BENCHMARK_RUN_INDEX_SCHEMA_V1,
    BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1,
};

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

/// Validate the stable paper-facing citation for one permanent protocol.
///
/// # Errors
/// Returns [`BenchError`] when protocol identity, publication status, or limitations conflict.
pub fn validate_benchmark_protocol_citation(
    citation: &BenchmarkProtocolCitation,
) -> Result<(), BenchError> {
    if citation.schema_version != BENCHMARK_PROTOCOL_CITATION_SCHEMA_V1
        || citation.citation_key != format!("bijux-benchmark-{}", citation.run.run_id)
        || citation.protocol_title.trim().is_empty()
        || citation.protocol_version.trim().is_empty()
        || !valid_sha256(&citation.protocol_sha256)
    {
        return Err(BenchError::InvalidPolicy("benchmark protocol citation is incomplete".into()));
    }
    benchmark_run_directory_name(&citation.run)?;
    match (citation.status, citation.selected_publication.as_ref()) {
        (BenchmarkProtocolCitationStatus::ProtocolOnly, None)
            if !citation.limitations.is_empty() => {}
        (BenchmarkProtocolCitationStatus::Published, Some(publication)) => {
            validate_selected_publication(
                publication,
                &citation.run.run_id,
                &citation.run.stage_id,
            )?;
        }
        _ => {
            return Err(BenchError::InvalidPolicy(
                "benchmark protocol citation publication status conflicts with its selection"
                    .into(),
            ));
        }
    }
    Ok(())
}

/// Validate the immutable case references for one permanent protocol.
///
/// # Errors
/// Returns [`BenchError`] when identity, ordering, case references, or limitations are invalid.
pub fn validate_benchmark_run_case_index(index: &BenchmarkRunCaseIndex) -> Result<(), BenchError> {
    if index.schema_version != BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1
        || !valid_sha256(&index.protocol_sha256)
        || (index.cases.is_empty() && index.limitations.is_empty())
    {
        return Err(BenchError::InvalidPolicy("benchmark run case index is incomplete".into()));
    }
    benchmark_run_directory_name(&index.run)?;
    let mut previous = None;
    let mut case_ids = BTreeSet::new();
    for case in &index.cases {
        if case.case_id.trim().is_empty()
            || case.tool_id.trim().is_empty()
            || previous.is_some_and(|value| value >= case.case_id.as_str())
            || !case_ids.insert(case.case_id.as_str())
        {
            return Err(BenchError::InvalidPolicy(
                "benchmark run case index identities are invalid or unsorted".into(),
            ));
        }
        validate_reference(
            &case.definition.uri,
            &case.definition.sha256,
            case.definition.size_bytes,
        )?;
        previous = Some(case.case_id.as_str());
    }
    Ok(())
}

/// Validate the current publication pointer for one permanent protocol.
///
/// # Errors
/// Returns [`BenchError`] when identity, publication closure, or unpublished limitations conflict.
pub fn validate_benchmark_run_publication_pointer(
    pointer: &BenchmarkRunPublicationPointer,
) -> Result<(), BenchError> {
    if pointer.schema_version != BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1
        || !valid_sha256(&pointer.protocol_sha256)
        || (pointer.selected_publication.is_none() && pointer.limitations.is_empty())
    {
        return Err(BenchError::InvalidPolicy(
            "benchmark run publication pointer is incomplete".into(),
        ));
    }
    benchmark_run_directory_name(&pointer.run)?;
    if let Some(publication) = &pointer.selected_publication {
        validate_selected_publication(publication, &pointer.run.run_id, &pointer.run.stage_id)?;
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
            validate_selected_publication(publication, &run.run_id, &run.stage_id)?;
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

fn validate_selected_publication(
    publication: &crate::model::BenchmarkRunIndexPublication,
    run_id: &str,
    stage_id: &str,
) -> Result<(), BenchError> {
    if publication.publication_id.trim().is_empty()
        || publication.selected_attempts.is_empty()
        || publication.repository_commit.len() != 40
        || !publication.repository_commit.bytes().all(|byte| byte.is_ascii_hexdigit())
        || publication.access_statement.trim().is_empty()
    {
        return Err(BenchError::InvalidPolicy(
            "benchmark selected publication is incomplete".into(),
        ));
    }
    for attempt in &publication.selected_attempts {
        validate_attempt_identity(attempt)?;
        if attempt.run.run_id != run_id || attempt.run.stage_id != stage_id {
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
