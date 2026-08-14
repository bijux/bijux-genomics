//! Owner: bijux-dna-bench
//! Reviewer-facing evidence books derived from admitted publication receipts.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_evidence_book, validate_benchmark_publication_receipt,
    validate_benchmark_run_index, BenchmarkEvidenceBook, BenchmarkEvidenceBookEntry,
    BenchmarkEvidenceBookStatus, BenchmarkEvidenceReference, BenchmarkPublicationReceipt,
    BenchmarkRunIndex, BenchmarkRunIndexEntry, BenchmarkRunIndexFileReference,
    BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1,
};

const RUN_INDEX_PATH: &str = "benchmarks/runs/RUN_INDEX.json";
const MAX_DOCUMENT_BYTES: usize = 32 * 1024 * 1024;

/// Inputs for one immutable evidence-book publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkEvidenceBookRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub book_id: String,
    pub generated_at_utc: String,
    pub publication_receipt_uris: Vec<String>,
    pub limitations: Vec<String>,
    pub output: PathBuf,
}

/// Build an evidence book from current selections plus explicitly admitted candidate receipts.
///
/// Explicit receipts let a newly admitted publication enter an immutable candidate evidence book
/// before its final checked-in pointer exists, removing the pointer/book dependency cycle. Every
/// receipt is reloaded from its immutable STUDY URI and checked against the permanent protocol.
/// Explicit candidates take precedence for their run; existing byte-identical books are accepted,
/// while a different book cannot replace one in place.
///
/// # Errors
/// Returns an error when source or STUDY authorities are invalid, a selected receipt is stale or
/// malformed, no admitted publication exists, or immutable output publication conflicts.
pub fn publish_checked_in_benchmark_evidence_book(
    request: &BenchmarkEvidenceBookRequest,
) -> Result<BenchmarkEvidenceBook> {
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize benchmark evidence-book repository")?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&repository_root)
            .context("require a clean benchmark evidence-book repository")?;
    let catalog_sha256 = bijux_dna_containers::api::catalog_identity_at_commit(
        &repository_root,
        &repository_commit,
    )?;
    let study_root = request
        .study_root
        .canonicalize()
        .context("canonicalize benchmark evidence-book STUDY root")?;
    let output = governed_output_path(&study_root, &request.output)?;
    let index: BenchmarkRunIndex =
        read_json(&repository_root.join(RUN_INDEX_PATH), "benchmark run index")?;
    validate_benchmark_run_index(&index)
        .context("validate benchmark run index for evidence book")?;
    let explicit_receipts = load_explicit_receipts(&study_root, &request.publication_receipt_uris)?;
    for run_id in explicit_receipts.keys() {
        if !index.runs.iter().any(|run| &run.run_id == run_id) {
            bail!("explicit benchmark publication receipt is absent from the permanent run index");
        }
    }

    let mut entries = Vec::new();
    let mut missing_run_ids = Vec::new();
    for run in &index.runs {
        if let Some((reference, receipt)) = explicit_receipts.get(&run.run_id) {
            validate_receipt_for_run(&repository_root, receipt, run)?;
            entries.push(evidence_book_entry(run, receipt, reference));
            continue;
        }
        let Some(selected) = &run.selected_publication else {
            missing_run_ids.push(run.run_id.clone());
            continue;
        };
        let receipt = load_indexed_receipt(&study_root, &selected.publication_receipt)?;
        if receipt.publication_id != selected.publication_id
            || receipt.run.run_id != run.run_id
            || receipt.repository_commit != selected.repository_commit
            || receipt.selected_attempts != selected.selected_attempts
        {
            bail!("benchmark publication receipt differs from its selected run-index identity");
        }
        validate_receipt_for_run(&repository_root, &receipt, run)?;
        entries.push(evidence_book_entry(
            run,
            &receipt,
            &BenchmarkEvidenceReference {
                uri: selected.publication_receipt.uri.clone(),
                sha256: selected.publication_receipt.sha256.clone(),
                size_bytes: selected.publication_receipt.size_bytes,
            },
        ));
    }
    let status = if missing_run_ids.is_empty() {
        BenchmarkEvidenceBookStatus::Complete
    } else {
        BenchmarkEvidenceBookStatus::Incomplete
    };
    let book = BenchmarkEvidenceBook {
        schema_version: BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1.to_string(),
        book_id: request.book_id.clone(),
        generated_at_utc: request.generated_at_utc.clone(),
        repository_commit,
        catalog_sha256,
        status,
        entries,
        missing_run_ids,
        limitations: request.limitations.clone(),
    };
    validate_benchmark_evidence_book(&book).context("validate benchmark evidence book")?;
    publish_identity(&output, &book)?;
    Ok(book)
}

fn load_explicit_receipts(
    study_root: &Path,
    uris: &[String],
) -> Result<BTreeMap<String, (BenchmarkEvidenceReference, BenchmarkPublicationReceipt)>> {
    let mut receipts = BTreeMap::new();
    for uri in uris {
        let reference = study_reference(study_root, uri, "explicit benchmark publication receipt")?;
        let indexed_reference = BenchmarkRunIndexFileReference {
            uri: reference.uri.clone(),
            sha256: reference.sha256.clone(),
            size_bytes: reference.size_bytes,
        };
        let receipt = load_indexed_receipt(study_root, &indexed_reference)?;
        if receipts.insert(receipt.run.run_id.clone(), (reference, receipt)).is_some() {
            bail!("more than one explicit benchmark publication receipt targets the same run");
        }
    }
    Ok(receipts)
}

fn validate_receipt_for_run(
    repository_root: &Path,
    receipt: &BenchmarkPublicationReceipt,
    run: &BenchmarkRunIndexEntry,
) -> Result<()> {
    if receipt.run.run_id != run.run_id
        || receipt.run.stage_id != run.stage_id
        || receipt.run_definition_sha256 != run.protocol_sha256
    {
        bail!("benchmark publication receipt differs from its permanent run protocol");
    }
    let source_catalog = bijux_dna_containers::api::catalog_identity_at_commit(
        repository_root,
        &receipt.repository_commit,
    )?;
    if receipt.catalog_sha256 != source_catalog {
        bail!("benchmark publication receipt catalog differs from its source commit");
    }
    Ok(())
}

fn evidence_book_entry(
    run: &BenchmarkRunIndexEntry,
    receipt: &BenchmarkPublicationReceipt,
    reference: &BenchmarkEvidenceReference,
) -> BenchmarkEvidenceBookEntry {
    BenchmarkEvidenceBookEntry {
        publication_id: receipt.publication_id.clone(),
        run: receipt.run.clone(),
        citation_key: run.citation_key.clone(),
        publication_receipt: reference.clone(),
    }
}

pub(super) fn governed_output_path(study_root: &Path, output: &Path) -> Result<PathBuf> {
    let file_name = output
        .file_name()
        .filter(|name| !name.is_empty())
        .context("benchmark evidence-book output has no file name")?;
    let parent = output
        .parent()
        .context("benchmark evidence-book output has no parent")?
        .canonicalize()
        .context("canonicalize benchmark evidence-book output parent")?;
    if !parent.starts_with(study_root)
        || output.components().any(|component| matches!(component, Component::ParentDir))
    {
        bail!("benchmark evidence-book output escapes the governed STUDY root");
    }
    Ok(parent.join(file_name))
}

pub(super) fn load_indexed_receipt(
    study_root: &Path,
    reference: &BenchmarkRunIndexFileReference,
) -> Result<BenchmarkPublicationReceipt> {
    let relative = reference
        .uri
        .strip_prefix("study://")
        .context("benchmark publication receipt is not a STUDY URI")?;
    let relative_path = Path::new(relative);
    if relative_path.as_os_str().is_empty()
        || relative_path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("benchmark publication receipt STUDY URI is unsafe");
    }
    let path = study_root.join(relative_path);
    let metadata = fs::symlink_metadata(&path)
        .with_context(|| format!("inspect benchmark publication receipt {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("benchmark publication receipt is not a regular file");
    }
    if metadata.len() != reference.size_bytes {
        bail!("benchmark publication receipt size differs from the run index");
    }
    if bijux_dna_infra::hash_file_sha256(&path)? != reference.sha256 {
        bail!("benchmark publication receipt digest differs from the run index");
    }
    let receipt: BenchmarkPublicationReceipt = read_json(&path, "benchmark publication receipt")?;
    validate_benchmark_publication_receipt(&receipt)
        .context("validate selected benchmark publication receipt")?;
    Ok(receipt)
}

pub(super) fn study_reference(
    study_root: &Path,
    uri: &str,
    label: &str,
) -> Result<BenchmarkEvidenceReference> {
    let path = study_path(study_root, uri, label)?;
    let metadata = fs::symlink_metadata(&path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular file");
    }
    Ok(BenchmarkEvidenceReference {
        uri: uri.to_string(),
        sha256: bijux_dna_infra::hash_file_sha256(&path)?,
        size_bytes: metadata.len(),
    })
}

pub(super) fn verify_study_reference(
    study_root: &Path,
    reference: &BenchmarkEvidenceReference,
    label: &str,
) -> Result<()> {
    let observed = study_reference(study_root, &reference.uri, label)?;
    if &observed != reference {
        bail!("{label} differs from its immutable evidence reference");
    }
    Ok(())
}

fn study_path(study_root: &Path, uri: &str, label: &str) -> Result<PathBuf> {
    let relative =
        uri.strip_prefix("study://").with_context(|| format!("{label} is not a STUDY URI"))?;
    let relative_path = Path::new(relative);
    if relative_path.as_os_str().is_empty()
        || relative_path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("{label} STUDY URI is unsafe");
    }
    Ok(study_root.join(relative_path))
}

pub(super) fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular file");
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_DOCUMENT_BYTES)
        .with_context(|| format!("read {label}"))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {label}"))
}

fn publish_identity(output: &Path, book: &BenchmarkEvidenceBook) -> Result<()> {
    let mut expected =
        serde_json::to_vec_pretty(book).context("serialize benchmark evidence book")?;
    expected.push(b'\n');
    match fs::symlink_metadata(output) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark evidence-book output is not a regular file");
            }
            if metadata.len() > MAX_DOCUMENT_BYTES as u64 {
                bail!("existing benchmark evidence book exceeds the bounded size limit");
            }
            if fs::read(output)? != expected {
                bail!("refusing to replace a different benchmark evidence book");
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(output, &expected)?;
            Ok(())
        }
        Err(error) => Err(error).context("inspect benchmark evidence-book output"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_book_output_must_remain_under_study() -> Result<()> {
        let temporary = bijux_dna_infra::temp_dir("benchmark-evidence-book-output")?;
        let study = temporary.path().join("study");
        let books = study.join("benchmarks/evidence-books");
        let outside = temporary.path().join("outside");
        fs::create_dir_all(&books)?;
        fs::create_dir(&outside)?;
        let study = study.canonicalize()?;

        assert_eq!(
            governed_output_path(&study, &books.join("book.json"))?,
            study.join("benchmarks/evidence-books/book.json")
        );
        assert!(governed_output_path(&study, &outside.join("book.json")).is_err());
        assert!(governed_output_path(&study, &books.join("../book.json")).is_err());
        Ok(())
    }

    #[test]
    fn publication_receipt_uri_rejects_parent_traversal_before_io() -> Result<()> {
        let temporary = bijux_dna_infra::temp_dir("benchmark-evidence-book-uri")?;
        let reference = BenchmarkRunIndexFileReference {
            uri: "study://benchmarks/../secret.json".to_string(),
            sha256: "a".repeat(64),
            size_bytes: 1,
        };
        assert!(load_indexed_receipt(temporary.path(), &reference).is_err());
        Ok(())
    }
}
