//! Validation for immutable benchmark publications and evidence books.

use std::collections::BTreeSet;

use chrono::DateTime;

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkAttemptIdentity, BenchmarkCitationRecord, BenchmarkEvidenceBook,
    BenchmarkEvidenceBookStatus, BenchmarkEvidenceReference, BenchmarkPublicationReceipt,
    BenchmarkRunIdentity,
};

use super::{
    benchmark_run_directory_name, validate_attempt_identity, BENCHMARK_CITATION_RECORD_SCHEMA_V1,
    BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1, BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1,
};

/// Validate one immutable publication selection and its full source/evidence closure.
///
/// # Errors
/// Returns an error when run, case, attempt, source-lock, report, decision, evidence, or limitation
/// identity is incomplete or inconsistent.
pub fn validate_benchmark_publication_receipt(
    receipt: &BenchmarkPublicationReceipt,
) -> Result<(), BenchError> {
    if receipt.schema_version != BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1 {
        return Err(invalid("benchmark publication-receipt schema mismatch"));
    }
    validate_component(&receipt.publication_id, "publication_id")?;
    validate_utc_timestamp(&receipt.published_at_utc, "published_at_utc")?;
    benchmark_run_directory_name(&receipt.run)?;
    validate_commit(&receipt.repository_commit)?;
    for digest in [
        receipt.catalog_sha256.as_str(),
        receipt.run_definition_sha256.as_str(),
        receipt.campaign_registry_sha256.as_str(),
    ] {
        validate_digest(digest)?;
    }
    validate_sorted_components(&receipt.case_ids, "case_ids", false)?;
    validate_attempt_selection(&receipt.run, &receipt.selected_attempts, "publication")?;
    validate_case_selection(&receipt.case_ids, &receipt.selected_attempts)?;
    validate_evidence_reference(&receipt.report)?;
    validate_evidence_reference(&receipt.decision)?;
    if receipt.report.uri == receipt.decision.uri {
        return Err(invalid("benchmark publication report and decision reuse one evidence URI"));
    }
    validate_evidence_set(&receipt.evidence)?;
    validate_limitations(&receipt.limitations)?;
    Ok(())
}

fn validate_case_selection(
    case_ids: &[String],
    attempts: &[BenchmarkAttemptIdentity],
) -> Result<(), BenchError> {
    let selected = attempts
        .iter()
        .map(|attempt| {
            attempt.case_id.as_deref().ok_or_else(|| {
                invalid("benchmark publication selected attempt omits its case identity")
            })
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let declared = case_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if attempts.len() != case_ids.len() || selected != declared {
        return Err(invalid("benchmark publication case identities differ from selected attempts"));
    }
    Ok(())
}

/// Validate one stable paper-facing benchmark citation.
///
/// # Errors
/// Returns an error when protocol, publication, attempt, tool/image, asset, evidence, limitation,
/// or access identity is incomplete or inconsistent.
pub fn validate_benchmark_citation_record(
    record: &BenchmarkCitationRecord,
) -> Result<(), BenchError> {
    if record.schema_version != BENCHMARK_CITATION_RECORD_SCHEMA_V1 {
        return Err(invalid("benchmark citation-record schema mismatch"));
    }
    benchmark_run_directory_name(&record.run)?;
    validate_component(&record.citation_key, "citation_key")?;
    validate_component(&record.publication_id, "publication_id")?;
    require_text(&record.protocol_title, "protocol_title", 10)?;
    require_text(&record.protocol_version, "protocol_version", 1)?;
    validate_digest(&record.protocol_sha256)?;
    validate_commit(&record.repository_commit)?;
    validate_attempt_selection(&record.run, &record.selected_attempts, "citation")?;
    validate_citation_tools(record)?;
    validate_citation_assets(record)?;
    for reference in [&record.report, &record.evidence_book, &record.publication_receipt] {
        validate_evidence_reference(reference)?;
    }
    let evidence_uris = [
        record.report.uri.as_str(),
        record.evidence_book.uri.as_str(),
        record.publication_receipt.uri.as_str(),
    ];
    if evidence_uris.windows(2).any(|pair| pair[0] == pair[1])
        || evidence_uris[0] == evidence_uris[2]
    {
        return Err(invalid("benchmark citation reuses one URI for distinct evidence roles"));
    }
    validate_limitations(&record.limitations)?;
    require_text(&record.access_statement, "access_statement", 20)?;
    Ok(())
}

/// Validate one reviewer-facing index of selected benchmark publications.
///
/// # Errors
/// Returns an error when source identity, completeness state, entry ordering, receipt references,
/// missing-run closure, or limitations are inconsistent.
pub fn validate_benchmark_evidence_book(book: &BenchmarkEvidenceBook) -> Result<(), BenchError> {
    if book.schema_version != BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1 {
        return Err(invalid("benchmark evidence-book schema mismatch"));
    }
    validate_component(&book.book_id, "book_id")?;
    validate_utc_timestamp(&book.generated_at_utc, "generated_at_utc")?;
    validate_commit(&book.repository_commit)?;
    validate_digest(&book.catalog_sha256)?;
    if book.entries.is_empty() {
        return Err(invalid("benchmark evidence book contains no publication entries"));
    }
    let mut publication_ids = BTreeSet::new();
    let mut citation_keys = BTreeSet::new();
    let mut run_ids = BTreeSet::new();
    let mut previous_run = None;
    for entry in &book.entries {
        benchmark_run_directory_name(&entry.run)?;
        validate_component(&entry.publication_id, "publication_id")?;
        validate_component(&entry.citation_key, "citation_key")?;
        validate_evidence_reference(&entry.publication_receipt)?;
        if previous_run.is_some_and(|run_id| run_id >= entry.run.run_id.as_str())
            || !run_ids.insert(entry.run.run_id.as_str())
            || !publication_ids.insert(entry.publication_id.as_str())
            || !citation_keys.insert(entry.citation_key.as_str())
        {
            return Err(invalid(
                "benchmark evidence-book entries must be run-sorted with unique publication and citation identity",
            ));
        }
        previous_run = Some(entry.run.run_id.as_str());
    }
    validate_run_ids(&book.missing_run_ids, &run_ids)?;
    match book.status {
        BenchmarkEvidenceBookStatus::Complete if !book.missing_run_ids.is_empty() => {
            return Err(invalid("complete benchmark evidence book declares missing runs"));
        }
        BenchmarkEvidenceBookStatus::Incomplete if book.missing_run_ids.is_empty() => {
            return Err(invalid("incomplete benchmark evidence book omits missing run identities"));
        }
        BenchmarkEvidenceBookStatus::Complete | BenchmarkEvidenceBookStatus::Incomplete => {}
    }
    validate_limitations(&book.limitations)?;
    Ok(())
}

pub(super) fn validate_attempt_selection(
    run: &BenchmarkRunIdentity,
    attempts: &[BenchmarkAttemptIdentity],
    context: &str,
) -> Result<(), BenchError> {
    if attempts.is_empty() {
        return Err(invalid(format!("benchmark {context} selects no attempts")));
    }
    let mut tools = BTreeSet::new();
    let mut previous_tool = None;
    let mut campaign_id = None;
    for attempt in attempts {
        validate_attempt_identity(attempt)?;
        if attempt.run != *run {
            return Err(invalid(format!("benchmark {context} attempt belongs to a different run")));
        }
        if campaign_id.is_some_and(|campaign| campaign != attempt.campaign_id.as_str()) {
            return Err(invalid(format!(
                "benchmark {context} attempts belong to different campaigns"
            )));
        }
        campaign_id = Some(attempt.campaign_id.as_str());
        if previous_tool.is_some_and(|tool| tool >= attempt.tool_id.as_str())
            || !tools.insert(attempt.tool_id.as_str())
        {
            return Err(invalid(format!(
                "benchmark {context} attempts must be tool-sorted with one attempt per tool"
            )));
        }
        previous_tool = Some(attempt.tool_id.as_str());
    }
    Ok(())
}

fn validate_citation_tools(record: &BenchmarkCitationRecord) -> Result<(), BenchError> {
    if record.tools.len() != record.selected_attempts.len() {
        return Err(invalid("benchmark citation tool closure differs from selected attempts"));
    }
    for (tool, attempt) in record.tools.iter().zip(&record.selected_attempts) {
        validate_component(&tool.tool_id, "tool_id")?;
        require_text(&tool.tool_version, "tool_version", 1)?;
        validate_digest(&tool.image_sha256)?;
        if tool.tool_id != attempt.tool_id {
            return Err(invalid(
                "benchmark citation tools do not align with selected attempt order",
            ));
        }
    }
    Ok(())
}

fn validate_citation_assets(record: &BenchmarkCitationRecord) -> Result<(), BenchError> {
    if record.assets.is_empty() {
        return Err(invalid("benchmark citation contains no immutable data assets"));
    }
    let mut assets = BTreeSet::new();
    let mut previous = None;
    for asset in &record.assets {
        validate_component(&asset.role, "asset role")?;
        validate_component(&asset.asset_id, "asset_id")?;
        validate_digest(&asset.sha256)?;
        let identity = (asset.role.as_str(), asset.asset_id.as_str());
        if previous.is_some_and(|value| value >= identity) || !assets.insert(identity) {
            return Err(invalid(
                "benchmark citation assets must be role-and-identity sorted and unique",
            ));
        }
        previous = Some(identity);
    }
    Ok(())
}

pub(super) fn validate_evidence_set(
    evidence: &[BenchmarkEvidenceReference],
) -> Result<(), BenchError> {
    if evidence.is_empty() {
        return Err(invalid("benchmark publication contains no supporting evidence"));
    }
    let mut uris = BTreeSet::new();
    let mut previous = None;
    for reference in evidence {
        validate_evidence_reference(reference)?;
        if previous.is_some_and(|uri| uri >= reference.uri.as_str())
            || !uris.insert(reference.uri.as_str())
        {
            return Err(invalid("benchmark publication evidence must be URI-sorted and unique"));
        }
        previous = Some(reference.uri.as_str());
    }
    Ok(())
}

pub(super) fn validate_evidence_reference(
    reference: &BenchmarkEvidenceReference,
) -> Result<(), BenchError> {
    validate_study_uri(&reference.uri)?;
    validate_digest(&reference.sha256)?;
    if reference.size_bytes == 0 {
        return Err(invalid("benchmark publication evidence has zero size"));
    }
    Ok(())
}

fn validate_run_ids(run_ids: &[String], published: &BTreeSet<&str>) -> Result<(), BenchError> {
    let mut previous = None;
    for run_id in run_ids {
        let valid = run_id
            .strip_prefix("run-")
            .and_then(|number| number.parse::<u16>().ok())
            .is_some_and(|number| (2000..3000).contains(&number));
        if !valid
            || published.contains(run_id.as_str())
            || previous.is_some_and(|value| value >= run_id.as_str())
        {
            return Err(invalid(
                "benchmark evidence-book missing runs must be canonical, sorted, unique, and absent from entries",
            ));
        }
        previous = Some(run_id.as_str());
    }
    Ok(())
}

pub(super) fn validate_limitations(limitations: &[String]) -> Result<(), BenchError> {
    if limitations.is_empty()
        || limitations.iter().any(|value| value.trim().len() < 10 || value != value.trim())
    {
        return Err(invalid("benchmark publication limitations are absent or malformed"));
    }
    Ok(())
}

fn validate_sorted_components(
    values: &[String],
    field: &str,
    allow_empty: bool,
) -> Result<(), BenchError> {
    if !allow_empty && values.is_empty() {
        return Err(invalid(format!("benchmark publication {field} cannot be empty")));
    }
    for value in values {
        validate_component(value, field)?;
    }
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid(format!("benchmark publication {field} must be sorted and unique")));
    }
    Ok(())
}

pub(super) fn validate_component(value: &str, field: &str) -> Result<(), BenchError> {
    if value.is_empty()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
        || value.starts_with(['-', '_'])
        || value.ends_with(['-', '_'])
    {
        return Err(invalid(format!("benchmark {field} is not a normalized component")));
    }
    Ok(())
}

fn validate_study_uri(uri: &str) -> Result<(), BenchError> {
    let Some(relative) = uri.strip_prefix("study://") else {
        return Err(invalid("benchmark publication evidence must use a study URI"));
    };
    if relative.is_empty()
        || relative.starts_with('/')
        || relative.ends_with('/')
        || relative.contains(['\\', '?', '#', '%'])
        || relative.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(invalid("benchmark publication evidence URI is not normalized"));
    }
    Ok(())
}

pub(super) fn validate_digest(value: &str) -> Result<(), BenchError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().any(|byte| byte.is_ascii_uppercase())
        || value.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid("benchmark publication SHA-256 identity is malformed"));
    }
    Ok(())
}

pub(super) fn validate_commit(value: &str) -> Result<(), BenchError> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().any(|byte| byte.is_ascii_uppercase())
        || value.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid("benchmark publication repository commit is malformed"));
    }
    Ok(())
}

pub(super) fn validate_utc_timestamp(value: &str, field: &str) -> Result<(), BenchError> {
    let timestamp = DateTime::parse_from_rfc3339(value)
        .map_err(|_| invalid(format!("benchmark publication {field} is not RFC 3339")))?;
    if timestamp.offset().local_minus_utc() != 0 {
        return Err(invalid(format!("benchmark publication {field} is not UTC")));
    }
    Ok(())
}

fn require_text(value: &str, field: &str, minimum_length: usize) -> Result<(), BenchError> {
    if value.trim().len() < minimum_length || value != value.trim() {
        return Err(invalid(format!("benchmark citation {field} is absent or malformed")));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
