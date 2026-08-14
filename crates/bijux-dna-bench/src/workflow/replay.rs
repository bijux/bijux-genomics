//! Owner: bijux-dna-bench
//! Semantic replay receipts for current benchmark publication selections.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    benchmark_run_definition_sha256, validate_benchmark_run_definition,
};
use bijux_dna_bench_model::{
    validate_benchmark_evidence_book, validate_benchmark_replay_receipt,
    validate_benchmark_run_index, BenchmarkEvidenceBook, BenchmarkEvidenceReference,
    BenchmarkPublicationReceipt, BenchmarkReplayOutcome, BenchmarkReplayReceipt,
    BenchmarkReplaySemanticIdentity, BenchmarkRunDefinition, BenchmarkRunIdentity,
    BenchmarkRunIndex, BenchmarkRunIndexEntry, BenchmarkRunIndexFileReference,
    BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1,
};

use super::evidence_book::{
    governed_output_path, load_indexed_receipt, read_json, study_reference, verify_study_reference,
};

const RUN_INDEX_PATH: &str = "benchmarks/runs/RUN_INDEX.json";
const MAX_REPLAY_RECEIPT_BYTES: u64 = 32 * 1024 * 1024;

/// Inputs for one semantic replay comparison against a selected publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkReplayRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub run_id: String,
    pub replay_id: String,
    pub replayed_at_utc: String,
    pub publication_receipt_uri: Option<String>,
    pub evidence_book_uri: Option<String>,
    pub run_definition_uri: String,
    pub case_definition_uris: Vec<String>,
    pub immutable_input_uris: Vec<String>,
    pub observed_decision_uri: Option<String>,
    pub observed_report_uri: Option<String>,
    pub observed_evidence_book_uri: Option<String>,
    pub differences: Vec<String>,
    pub limitations: Vec<String>,
    pub output: PathBuf,
}

/// Compare regenerated semantics with a selected or candidate publication and publish a receipt.
///
/// Scheduler identifiers and timing are intentionally excluded from semantic equality. All source,
/// input, case, and observed artifacts are independently hashed from governed STUDY paths. An
/// explicit receipt and candidate book may be used together before the final tracked pointer exists;
/// mixed explicit/indexed selection is rejected.
///
/// # Errors
/// Returns an error when the selected or candidate closure is invalid, observed semantics are
/// partial or contradictory, or immutable receipt publication conflicts.
pub fn replay_checked_in_benchmark_publication(
    request: &BenchmarkReplayRequest,
) -> Result<BenchmarkReplayReceipt> {
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize benchmark replay repository")?;
    let replay_repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&repository_root)
            .context("require a clean benchmark replay repository")?;
    let study_root =
        request.study_root.canonicalize().context("canonicalize benchmark replay STUDY root")?;
    let output = governed_output_path(&study_root, &request.output)?;
    let index: BenchmarkRunIndex =
        read_json(&repository_root.join(RUN_INDEX_PATH), "benchmark replay run index")?;
    validate_benchmark_run_index(&index).context("validate benchmark replay run index")?;
    let run = index
        .runs
        .iter()
        .find(|run| run.run_id == request.run_id)
        .context("benchmark replay run is absent from the run index")?;
    let ReplaySourceClosure { receipt: source, publication: source_publication, evidence_book } =
        load_replay_source(request, &study_root, run)?;
    let source_catalog = bijux_dna_containers::api::catalog_identity_at_commit(
        &repository_root,
        &source.repository_commit,
    )?;
    if source.catalog_sha256 != source_catalog {
        bail!("benchmark replay source catalog differs from its repository commit");
    }
    verify_study_reference(&study_root, &source.report, "benchmark replay source report")?;
    verify_study_reference(&study_root, &source.decision, "benchmark replay source decision")?;
    verify_study_reference(&study_root, &evidence_book, "benchmark replay source evidence book")?;

    let run_definition = study_reference(
        &study_root,
        &request.run_definition_uri,
        "benchmark replay run definition",
    )?;
    let relative = run_definition
        .uri
        .strip_prefix("study://")
        .context("validated benchmark run-definition URI lost its prefix")?;
    let definition: BenchmarkRunDefinition =
        read_json(&study_root.join(relative), "benchmark replay run definition")?;
    validate_replay_run_definition(&definition, &source.run, &source.run_definition_sha256)?;
    let case_definitions = request
        .case_definition_uris
        .iter()
        .map(|uri| study_reference(&study_root, uri, "benchmark replay case definition"))
        .collect::<Result<Vec<_>>>()?;
    let immutable_inputs = request
        .immutable_input_uris
        .iter()
        .map(|uri| study_reference(&study_root, uri, "benchmark replay immutable input"))
        .collect::<Result<Vec<_>>>()?;
    let expected = BenchmarkReplaySemanticIdentity {
        decision_sha256: source.decision.sha256,
        report_sha256: source.report.sha256,
        evidence_book_sha256: evidence_book.sha256,
    };
    let observed = observed_semantics(request, &study_root)?;
    let outcome = match &observed {
        Some(observed) if observed == &expected => BenchmarkReplayOutcome::ExactMatch,
        Some(_) => BenchmarkReplayOutcome::Mismatch,
        None => BenchmarkReplayOutcome::Blocked,
    };
    let differences =
        replay_differences(outcome, &expected, observed.as_ref(), &request.differences)?;
    let receipt = BenchmarkReplayReceipt {
        schema_version: BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1.to_string(),
        replay_id: request.replay_id.clone(),
        replayed_at_utc: request.replayed_at_utc.clone(),
        run: source.run,
        publication_id: source.publication_id,
        selected_attempts: source.selected_attempts,
        source_repository_commit: source.repository_commit,
        replay_repository_commit,
        source_publication,
        run_definition,
        case_definitions,
        immutable_inputs,
        expected,
        observed,
        outcome,
        differences,
        limitations: request.limitations.clone(),
    };
    validate_benchmark_replay_receipt(&receipt).context("validate benchmark replay receipt")?;
    publish_identity(&output, &receipt)?;
    Ok(receipt)
}

fn validate_replay_run_definition(
    definition: &BenchmarkRunDefinition,
    run: &BenchmarkRunIdentity,
    expected_sha256: &str,
) -> Result<()> {
    validate_benchmark_run_definition(definition)
        .context("validate benchmark replay run definition")?;
    if definition.identity != *run
        || benchmark_run_definition_sha256(definition)? != expected_sha256
    {
        bail!("benchmark replay run definition differs from the publication receipt");
    }
    Ok(())
}

struct ReplaySourceClosure {
    receipt: BenchmarkPublicationReceipt,
    publication: BenchmarkEvidenceReference,
    evidence_book: BenchmarkEvidenceReference,
}

fn load_replay_source(
    request: &BenchmarkReplayRequest,
    study_root: &Path,
    run: &BenchmarkRunIndexEntry,
) -> Result<ReplaySourceClosure> {
    match candidate_source_uris(
        request.publication_receipt_uri.as_deref(),
        request.evidence_book_uri.as_deref(),
    )? {
        Some((receipt_uri, book_uri)) => {
            load_explicit_replay_source(study_root, run, receipt_uri, book_uri)
        }
        None => load_selected_replay_source(study_root, run),
    }
}

fn candidate_source_uris<'a>(
    publication_receipt_uri: Option<&'a str>,
    evidence_book_uri: Option<&'a str>,
) -> Result<Option<(&'a str, &'a str)>> {
    match (publication_receipt_uri, evidence_book_uri) {
        (Some(receipt_uri), Some(book_uri)) => Ok(Some((receipt_uri, book_uri))),
        (None, None) => Ok(None),
        _ => bail!("benchmark replay candidate selection requires both receipt and evidence book"),
    }
}

fn load_selected_replay_source(
    study_root: &Path,
    run: &BenchmarkRunIndexEntry,
) -> Result<ReplaySourceClosure> {
    let selected = run
        .selected_publication
        .as_ref()
        .context("benchmark replay run has no selected publication")?;
    let receipt = load_indexed_receipt(study_root, &selected.publication_receipt)?;
    if receipt.publication_id != selected.publication_id
        || receipt.run.run_id != run.run_id
        || receipt.repository_commit != selected.repository_commit
        || receipt.selected_attempts != selected.selected_attempts
    {
        bail!("benchmark replay source receipt differs from the selected publication");
    }
    Ok(ReplaySourceClosure {
        receipt,
        publication: indexed_reference(&selected.publication_receipt),
        evidence_book: indexed_reference(&selected.evidence_book),
    })
}

fn load_explicit_replay_source(
    study_root: &Path,
    run: &BenchmarkRunIndexEntry,
    receipt_uri: &str,
    book_uri: &str,
) -> Result<ReplaySourceClosure> {
    let publication =
        study_reference(study_root, receipt_uri, "benchmark replay candidate publication receipt")?;
    let receipt = load_indexed_receipt(
        study_root,
        &BenchmarkRunIndexFileReference {
            uri: publication.uri.clone(),
            sha256: publication.sha256.clone(),
            size_bytes: publication.size_bytes,
        },
    )?;
    if receipt.run.run_id != run.run_id
        || receipt.run.stage_id != run.stage_id
        || receipt.run_definition_sha256 != run.protocol_sha256
    {
        bail!("benchmark replay candidate receipt differs from its permanent run protocol");
    }

    let evidence_book =
        study_reference(study_root, book_uri, "benchmark replay candidate evidence book")?;
    let relative = evidence_book
        .uri
        .strip_prefix("study://")
        .context("validated benchmark evidence-book URI lost its prefix")?;
    let book: BenchmarkEvidenceBook =
        read_json(&study_root.join(relative), "benchmark replay candidate evidence book")?;
    validate_benchmark_evidence_book(&book)
        .context("validate benchmark replay candidate evidence book")?;
    let entry = book
        .entries
        .iter()
        .find(|entry| entry.run.run_id == run.run_id)
        .context("benchmark replay candidate evidence book omits the requested run")?;
    if entry.publication_id != receipt.publication_id
        || entry.run != receipt.run
        || entry.publication_receipt != publication
    {
        bail!("benchmark replay candidate evidence book differs from its publication receipt");
    }
    Ok(ReplaySourceClosure { receipt, publication, evidence_book })
}

fn observed_semantics(
    request: &BenchmarkReplayRequest,
    study_root: &Path,
) -> Result<Option<BenchmarkReplaySemanticIdentity>> {
    match (
        request.observed_decision_uri.as_deref(),
        request.observed_report_uri.as_deref(),
        request.observed_evidence_book_uri.as_deref(),
    ) {
        (Some(decision), Some(report), Some(book)) => Ok(Some(BenchmarkReplaySemanticIdentity {
            decision_sha256: study_reference(
                study_root,
                decision,
                "benchmark replay observed decision",
            )?
            .sha256,
            report_sha256: study_reference(study_root, report, "benchmark replay observed report")?
                .sha256,
            evidence_book_sha256: study_reference(
                study_root,
                book,
                "benchmark replay observed evidence book",
            )?
            .sha256,
        })),
        (None, None, None) => Ok(None),
        _ => bail!("benchmark replay observed semantic closure is partial"),
    }
}

fn replay_differences(
    outcome: BenchmarkReplayOutcome,
    expected: &BenchmarkReplaySemanticIdentity,
    observed: Option<&BenchmarkReplaySemanticIdentity>,
    declared: &[String],
) -> Result<Vec<String>> {
    match outcome {
        BenchmarkReplayOutcome::ExactMatch => {
            if !declared.is_empty() {
                bail!("exact benchmark replay cannot declare semantic differences");
            }
            Ok(Vec::new())
        }
        BenchmarkReplayOutcome::Blocked => {
            if declared.is_empty() {
                bail!("blocked benchmark replay requires an actionable reason");
            }
            Ok(declared.to_vec())
        }
        BenchmarkReplayOutcome::Mismatch => {
            if !declared.is_empty() {
                return Ok(declared.to_vec());
            }
            let observed =
                observed.context("mismatched benchmark replay omits observed semantics")?;
            let mut differences = Vec::new();
            if observed.decision_sha256 != expected.decision_sha256 {
                differences.push("The regenerated scientific decision digest differs.".to_string());
            }
            if observed.report_sha256 != expected.report_sha256 {
                differences.push("The regenerated benchmark report digest differs.".to_string());
            }
            if observed.evidence_book_sha256 != expected.evidence_book_sha256 {
                differences.push("The regenerated evidence-book digest differs.".to_string());
            }
            Ok(differences)
        }
    }
}

fn indexed_reference(
    reference: &bijux_dna_bench_model::BenchmarkRunIndexFileReference,
) -> BenchmarkEvidenceReference {
    BenchmarkEvidenceReference {
        uri: reference.uri.clone(),
        sha256: reference.sha256.clone(),
        size_bytes: reference.size_bytes,
    }
}

fn publish_identity(output: &Path, receipt: &BenchmarkReplayReceipt) -> Result<()> {
    let mut expected =
        serde_json::to_vec_pretty(receipt).context("serialize benchmark replay receipt")?;
    expected.push(b'\n');
    match fs::symlink_metadata(output) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark replay-receipt output is not a regular file");
            }
            if metadata.len() > MAX_REPLAY_RECEIPT_BYTES {
                bail!("existing benchmark replay receipt exceeds the bounded size limit");
            }
            if fs::read(output)? != expected {
                bail!("refusing to replace a different benchmark replay receipt");
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(output, &expected)?;
            Ok(())
        }
        Err(error) => Err(error).context("inspect benchmark replay-receipt output"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn semantics(value: char) -> BenchmarkReplaySemanticIdentity {
        BenchmarkReplaySemanticIdentity {
            decision_sha256: value.to_string().repeat(64),
            report_sha256: value.to_string().repeat(64),
            evidence_book_sha256: value.to_string().repeat(64),
        }
    }

    #[test]
    fn exact_replay_refuses_declared_differences() {
        let expected = semantics('a');
        assert!(replay_differences(
            BenchmarkReplayOutcome::ExactMatch,
            &expected,
            Some(&expected),
            &["A difference cannot accompany an exact replay.".to_string()],
        )
        .is_err());
    }

    #[test]
    fn mismatch_names_each_changed_semantic_digest() -> Result<()> {
        let expected = semantics('a');
        let observed = semantics('b');
        let differences =
            replay_differences(BenchmarkReplayOutcome::Mismatch, &expected, Some(&observed), &[])?;
        assert_eq!(differences.len(), 3);
        Ok(())
    }

    #[test]
    fn candidate_replay_requires_receipt_and_book_together() -> Result<()> {
        assert!(candidate_source_uris(Some("study://receipt.json"), None).is_err());
        assert!(candidate_source_uris(None, Some("study://book.json")).is_err());
        assert_eq!(
            candidate_source_uris(Some("study://receipt.json"), Some("study://book.json"))?,
            Some(("study://receipt.json", "study://book.json"))
        );
        Ok(())
    }

    #[test]
    fn replay_compares_semantic_protocol_identity_not_file_formatting() -> Result<()> {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .context("benchmark replay crate is outside the repository")?;
        let path =
            repository_root.join("benchmarks/runs/run-2043-fastq-profile-reads/definition.json");
        let definition: BenchmarkRunDefinition =
            read_json(&path, "benchmark replay protocol fixture")?;
        let semantic_sha256 = benchmark_run_definition_sha256(&definition)?;
        assert_ne!(bijux_dna_infra::hash_file_sha256(&path)?, semantic_sha256);
        validate_replay_run_definition(&definition, &definition.identity, &semantic_sha256)?;
        Ok(())
    }
}
