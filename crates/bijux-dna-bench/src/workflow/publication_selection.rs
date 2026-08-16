//! Owner: bijux-dna-bench
//! Atomic checked-in selection of fully admitted benchmark publications.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_citation_record, validate_benchmark_evidence_book,
    validate_benchmark_replay_receipt, validate_benchmark_run_case_index,
    validate_benchmark_run_index, validate_benchmark_run_publication_pointer,
    BenchmarkCaseDefinitionV1, BenchmarkCitationRecord, BenchmarkEvidenceBook,
    BenchmarkEvidenceReference, BenchmarkPublicationReceipt, BenchmarkReplayOutcome,
    BenchmarkReplayReceipt, BenchmarkRunCaseIndex, BenchmarkRunCaseIndexEntry, BenchmarkRunIndex,
    BenchmarkRunIndexFileReference, BenchmarkRunIndexPublication, BenchmarkRunPublicationPointer,
    BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1, BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1,
};
use chrono::DateTime;

use super::evidence_book::{
    load_indexed_receipt, read_json, study_reference, verify_study_reference,
};
use super::publication_registry::{
    derive_benchmark_publication_registry, publish_checked_in_benchmark_publication_registry,
    BenchmarkPublicationRegistryWriteMode,
};
use super::run_index::{publish_checked_in_benchmark_run_index, BenchmarkRunIndexWriteMode};

const RUN_INDEX_PATH: &str = "benchmarks/runs/RUN_INDEX.json";

/// Immutable evidence authorities for one checked-in publication selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkPublicationSelectionRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub run_id: String,
    pub publication_receipt_uri: String,
    pub evidence_book_uri: String,
    pub citation_record_uri: String,
    pub replay_receipt_uri: String,
}

/// Select one publication and regenerate all paper-facing run and report-index surfaces.
///
/// Immutable publication artifacts remain untouched. The checked-in current pointer may advance
/// to a later publication whose source descends from the selected source; equal or older
/// publication timestamps and divergent source histories are rejected. Case definitions are
/// parsed and matched to selected attempts before the case index is written. The case index is
/// written before the pointer so interruption cannot expose a selected publication without its
/// case closure.
///
/// # Errors
/// Returns an error when any receipt, book, citation, replay, protocol, case, or existing selection
/// differs, or when generated run surfaces cannot be published.
pub fn select_checked_in_benchmark_publication(
    request: &BenchmarkPublicationSelectionRequest,
) -> Result<BenchmarkRunIndex> {
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize benchmark selection repository")?;
    bijux_dna_containers::api::clean_repository_head_commit(&repository_root)
        .context("require a clean benchmark selection repository")?;
    let study_root =
        request.study_root.canonicalize().context("canonicalize benchmark selection STUDY root")?;
    let index: BenchmarkRunIndex =
        read_json(&repository_root.join(RUN_INDEX_PATH), "benchmark selection run index")?;
    validate_benchmark_run_index(&index).context("validate benchmark selection run index")?;
    let run = index
        .runs
        .iter()
        .find(|run| run.run_id == request.run_id)
        .context("benchmark selection run is absent from the permanent index")?;

    let publication_reference = study_reference(
        &study_root,
        &request.publication_receipt_uri,
        "benchmark selection publication receipt",
    )?;
    let publication =
        load_indexed_receipt(&study_root, &indexed_reference(&publication_reference))?;
    if publication.run.run_id != run.run_id
        || publication.run.stage_id != run.stage_id
        || publication.run_definition_sha256 != run.protocol_sha256
    {
        bail!("benchmark selection receipt differs from the permanent protocol");
    }

    let book_reference = study_reference(
        &study_root,
        &request.evidence_book_uri,
        "benchmark selection evidence book",
    )?;
    let book: BenchmarkEvidenceBook =
        read_study_json(&study_root, &book_reference.uri, "benchmark selection evidence book")?;
    validate_benchmark_evidence_book(&book)
        .context("validate benchmark selection evidence book")?;
    let entry = book
        .entries
        .iter()
        .find(|entry| entry.run == publication.run)
        .context("benchmark selection evidence book omits the publication run")?;
    if entry.publication_id != publication.publication_id
        || entry.citation_key != run.citation_key
        || entry.publication_receipt != publication_reference
    {
        bail!("benchmark selection evidence book differs from the publication receipt");
    }

    let citation_reference = study_reference(
        &study_root,
        &request.citation_record_uri,
        "benchmark selection citation record",
    )?;
    let citation: BenchmarkCitationRecord = read_study_json(
        &study_root,
        &citation_reference.uri,
        "benchmark selection citation record",
    )?;
    validate_benchmark_citation_record(&citation)
        .context("validate benchmark selection citation record")?;
    validate_citation(&citation, run, &publication, &publication_reference, &book_reference)?;

    let replay_reference = study_reference(
        &study_root,
        &request.replay_receipt_uri,
        "benchmark selection replay receipt",
    )?;
    let replay: BenchmarkReplayReceipt =
        read_study_json(&study_root, &replay_reference.uri, "benchmark selection replay receipt")?;
    validate_benchmark_replay_receipt(&replay)
        .context("validate benchmark selection replay receipt")?;
    validate_replay(&replay, &publication, &publication_reference, &book_reference)?;

    let selected = BenchmarkRunIndexPublication {
        publication_id: publication.publication_id.clone(),
        selected_attempts: publication.selected_attempts.clone(),
        repository_commit: publication.repository_commit.clone(),
        report: indexed_reference(&publication.report),
        evidence_book: indexed_reference(&book_reference),
        publication_receipt: indexed_reference(&publication_reference),
        citation_record: indexed_reference(&citation_reference),
        replay_receipt: indexed_reference(&replay_reference),
        limitations: citation.limitations.clone(),
        access_statement: citation.access_statement.clone(),
    };
    let pointer = BenchmarkRunPublicationPointer {
        schema_version: BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1.to_string(),
        run: publication.run.clone(),
        protocol_sha256: publication.run_definition_sha256.clone(),
        selected_publication: Some(selected),
        limitations: citation.limitations,
    };
    validate_benchmark_run_publication_pointer(&pointer)
        .context("validate benchmark publication selection pointer")?;
    let case_index = derive_case_index(&study_root, &publication, &replay)?;

    let run_directory =
        repository_root.join("benchmarks/runs").join(format!("{}-{}", run.run_id, run.slug));
    let case_index_path = run_directory.join("case-index.json");
    let pointer_path = run_directory.join("current-publication.json");
    let supersedes_selection = validate_existing_pointer(
        &repository_root,
        &study_root,
        &pointer_path,
        &pointer,
        &publication,
    )?;
    validate_existing_case_index(&case_index_path, &case_index, supersedes_selection)?;
    preflight_publication_selection(&repository_root)
        .context("preflight generated benchmark publication surfaces")?;
    write_json(&case_index_path, &case_index, "benchmark run case index")?;
    write_json(&pointer_path, &pointer, "benchmark current publication pointer")?;
    let index = publish_checked_in_benchmark_run_index(
        &repository_root,
        BenchmarkRunIndexWriteMode::Write,
    )?;
    publish_checked_in_benchmark_publication_registry(
        &repository_root,
        BenchmarkPublicationRegistryWriteMode::Write,
    )
    .context("regenerate benchmark report index after publication selection")?;
    Ok(index)
}

fn preflight_publication_selection(repository_root: &Path) -> Result<()> {
    publish_checked_in_benchmark_run_index(repository_root, BenchmarkRunIndexWriteMode::Check)
        .context("preflight benchmark run index")?;
    derive_benchmark_publication_registry(repository_root)
        .context("preflight benchmark publication registry")?;
    Ok(())
}

fn validate_citation(
    citation: &BenchmarkCitationRecord,
    run: &bijux_dna_bench_model::BenchmarkRunIndexEntry,
    publication: &BenchmarkPublicationReceipt,
    publication_reference: &BenchmarkEvidenceReference,
    book_reference: &BenchmarkEvidenceReference,
) -> Result<()> {
    if citation.citation_key != run.citation_key
        || citation.run != publication.run
        || citation.protocol_sha256 != publication.run_definition_sha256
        || citation.publication_id != publication.publication_id
        || citation.selected_attempts != publication.selected_attempts
        || citation.repository_commit != publication.repository_commit
        || citation.report != publication.report
        || citation.evidence_book != *book_reference
        || citation.publication_receipt != *publication_reference
    {
        bail!("benchmark selection citation differs from the publication closure");
    }
    Ok(())
}

fn validate_replay(
    replay: &BenchmarkReplayReceipt,
    publication: &BenchmarkPublicationReceipt,
    publication_reference: &BenchmarkEvidenceReference,
    book_reference: &BenchmarkEvidenceReference,
) -> Result<()> {
    if replay.outcome != BenchmarkReplayOutcome::ExactMatch
        || replay.run != publication.run
        || replay.publication_id != publication.publication_id
        || replay.selected_attempts != publication.selected_attempts
        || replay.source_repository_commit != publication.repository_commit
        || replay.source_publication != *publication_reference
        || replay.expected.report_sha256 != publication.report.sha256
        || replay.expected.evidence_book_sha256 != book_reference.sha256
    {
        bail!("benchmark selection replay differs from the publication closure");
    }
    Ok(())
}

fn derive_case_index(
    study_root: &Path,
    publication: &BenchmarkPublicationReceipt,
    replay: &BenchmarkReplayReceipt,
) -> Result<BenchmarkRunCaseIndex> {
    let mut cases = publication
        .selected_attempts
        .iter()
        .zip(&replay.case_definitions)
        .map(|(attempt, reference)| {
            verify_study_reference(study_root, reference, "selected benchmark case definition")?;
            let definition: BenchmarkCaseDefinitionV1 =
                read_study_json(study_root, &reference.uri, "selected benchmark case definition")?;
            let case_id = attempt
                .case_id
                .as_deref()
                .context("selected benchmark attempt omits its case identity")?;
            if definition.case_id != case_id
                || definition.run_id != attempt.run.run_id
                || definition.domain != attempt.run.domain
                || definition.stage_id != attempt.run.stage_id
                || definition.tool_id != attempt.tool_id
            {
                bail!("selected benchmark case definition differs from its attempt");
            }
            Ok(BenchmarkRunCaseIndexEntry {
                case_id: case_id.to_string(),
                tool_id: attempt.tool_id.clone(),
                definition: indexed_reference(reference),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    let index = BenchmarkRunCaseIndex {
        schema_version: BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1.to_string(),
        run: publication.run.clone(),
        protocol_sha256: publication.run_definition_sha256.clone(),
        cases,
        limitations: publication.limitations.clone(),
    };
    validate_benchmark_run_case_index(&index).context("validate selected benchmark case index")?;
    Ok(index)
}

fn validate_existing_case_index(
    path: &Path,
    expected: &BenchmarkRunCaseIndex,
    supersedes_selection: bool,
) -> Result<()> {
    let current: BenchmarkRunCaseIndex = read_json(path, "existing benchmark case index")?;
    validate_benchmark_run_case_index(&current)?;
    if current.run != expected.run || current.protocol_sha256 != expected.protocol_sha256 {
        bail!("existing benchmark case index differs from the permanent protocol");
    }
    if !current.cases.is_empty() && current != *expected && !supersedes_selection {
        bail!("refusing to replace a different populated benchmark case index");
    }
    Ok(())
}

fn validate_existing_pointer(
    repository_root: &Path,
    study_root: &Path,
    path: &Path,
    expected: &BenchmarkRunPublicationPointer,
    incoming_receipt: &BenchmarkPublicationReceipt,
) -> Result<bool> {
    let current: BenchmarkRunPublicationPointer =
        read_json(path, "existing benchmark publication pointer")?;
    validate_benchmark_run_publication_pointer(&current)?;
    if current.run != expected.run || current.protocol_sha256 != expected.protocol_sha256 {
        bail!("existing benchmark publication pointer differs from the permanent protocol");
    }
    if current == *expected {
        return Ok(false);
    }
    let Some(previous) = current.selected_publication else {
        return Ok(true);
    };
    let next = expected
        .selected_publication
        .as_ref()
        .context("incoming benchmark publication pointer omits its selection")?;
    if !bijux_dna_containers::api::repository_commit_is_ancestor(
        repository_root,
        &previous.repository_commit,
        &next.repository_commit,
    )? {
        bail!("benchmark publication selection source does not descend from the current source");
    }
    let previous_receipt = load_indexed_receipt(study_root, &previous.publication_receipt)?;
    if previous_receipt.publication_id != previous.publication_id
        || previous_receipt.run != current.run
        || previous_receipt.repository_commit != previous.repository_commit
        || previous_receipt.selected_attempts != previous.selected_attempts
    {
        bail!("current benchmark publication pointer differs from its immutable receipt");
    }
    validate_publication_order(
        &previous_receipt.published_at_utc,
        &incoming_receipt.published_at_utc,
    )?;
    Ok(true)
}

fn validate_publication_order(previous: &str, incoming: &str) -> Result<()> {
    let previous = DateTime::parse_from_rfc3339(previous)
        .context("parse current benchmark publication timestamp")?;
    let incoming = DateTime::parse_from_rfc3339(incoming)
        .context("parse incoming benchmark publication timestamp")?;
    if incoming <= previous {
        bail!("benchmark publication selection does not advance publication time");
    }
    Ok(())
}

fn write_json(path: &Path, value: &impl serde::Serialize, label: &str) -> Result<()> {
    let mut bytes =
        serde_json::to_vec_pretty(value).with_context(|| format!("serialize {label}"))?;
    bytes.push(b'\n');
    bijux_dna_infra::atomic_write_bytes(path, &bytes)
        .with_context(|| format!("write {label} {}", path.display()))
}

fn indexed_reference(reference: &BenchmarkEvidenceReference) -> BenchmarkRunIndexFileReference {
    BenchmarkRunIndexFileReference {
        uri: reference.uri.clone(),
        sha256: reference.sha256.clone(),
        size_bytes: reference.size_bytes,
    }
}

fn read_study_json<T: serde::de::DeserializeOwned>(
    study_root: &Path,
    uri: &str,
    label: &str,
) -> Result<T> {
    let reference = study_reference(study_root, uri, label)?;
    let relative = reference
        .uri
        .strip_prefix("study://")
        .context("validated benchmark selection URI lost its prefix")?;
    read_json(&study_root.join(relative), label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publication_selection_preflights_every_generated_surface_before_writing() -> Result<()> {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .context("benchmark selection crate is outside the repository")?;

        preflight_publication_selection(repository_root)
    }

    #[test]
    fn publication_selection_order_rejects_stale_and_equal_receipts() -> Result<()> {
        validate_publication_order("2026-08-14T13:53:00Z", "2026-08-14T13:54:00Z")?;
        assert!(validate_publication_order("2026-08-14T13:53:00Z", "2026-08-14T13:53:00Z").is_err());
        assert!(validate_publication_order("2026-08-14T13:53:00Z", "2026-08-14T13:52:59Z").is_err());
        Ok(())
    }

    #[test]
    fn populated_case_index_requires_an_authorized_selection_supersession() -> Result<()> {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .context("benchmark selection crate is outside the repository")?;
        let source = repository_root
            .join("benchmarks/runs/run-2042-fastq-profile-read-lengths/current-publication.json");
        let pointer: BenchmarkRunPublicationPointer =
            read_json(&source, "benchmark selection pointer fixture")?;
        let case_source = repository_root
            .join("benchmarks/runs/run-2042-fastq-profile-read-lengths/case-index.json");
        let expected: BenchmarkRunCaseIndex =
            read_json(&case_source, "benchmark case-index fixture")?;
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("case-index.json");
        write_json(&path, &expected, "benchmark case-index fixture")?;
        validate_existing_case_index(&path, &expected, false)?;
        let mut conflicting = expected.clone();
        conflicting.limitations = vec!["A newer admitted publication has different cases.".into()];
        assert!(validate_existing_case_index(&path, &conflicting, false).is_err());
        validate_existing_case_index(&path, &conflicting, true)?;
        assert_eq!(pointer.run, expected.run);
        Ok(())
    }
}
