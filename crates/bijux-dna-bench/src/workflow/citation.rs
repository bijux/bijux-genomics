//! Owner: bijux-dna-bench
//! Paper-facing citation records derived from admitted benchmark publication closure.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    benchmark_run_definition_sha256, validate_benchmark_run_definition, validate_stage_report,
};
use bijux_dna_bench_model::{
    validate_benchmark_citation_record, validate_benchmark_evidence_book,
    validate_benchmark_replay_receipt, validate_benchmark_run_index,
    BenchmarkCitationAssetIdentity, BenchmarkCitationRecord, BenchmarkCitationToolIdentity,
    BenchmarkDataSnapshotRoleV1, BenchmarkDataSnapshotV1, BenchmarkEvidenceBook,
    BenchmarkEvidenceReference, BenchmarkPublicationReceipt, BenchmarkReplayOutcome,
    BenchmarkReplayReceipt, BenchmarkRunDefinition, BenchmarkRunIndex, BenchmarkRunIndexEntry,
    BenchmarkRunIndexFileReference, BenchmarkStageReport, BENCHMARK_CITATION_RECORD_SCHEMA_V1,
};

use super::campaign_engine::validate_benchmark_case_matrix;
use super::evidence_book::{
    governed_output_path, load_indexed_receipt, read_json, study_reference, verify_study_reference,
};

const RUN_INDEX_PATH: &str = "benchmarks/runs/RUN_INDEX.json";
const MAX_CITATION_BYTES: u64 = 32 * 1024 * 1024;

/// Inputs for one immutable, paper-facing benchmark citation record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCitationRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub publication_receipt_uri: String,
    pub evidence_book_uri: String,
    pub replay_receipt_uri: String,
    pub case_matrix_uri: String,
    pub limitations: Vec<String>,
    pub access_statement: String,
    pub output: PathBuf,
}

/// Derive a citation only from an exact admitted publication, replay, image, and data closure.
///
/// Scheduler identifiers and mutable paths are intentionally excluded. Tool versions come from
/// the publication source commit's validated catalog, image digests come from the admitted report,
/// and data identities come from the source-locked case matrix.
///
/// # Errors
/// Returns an error when any source, publication, replay, tool, image, or data authority differs.
pub fn issue_benchmark_citation_record(
    request: &BenchmarkCitationRequest,
) -> Result<BenchmarkCitationRecord> {
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize benchmark citation repository")?;
    bijux_dna_containers::api::clean_repository_head_commit(&repository_root)
        .context("require a clean benchmark citation repository")?;
    let study_root =
        request.study_root.canonicalize().context("canonicalize benchmark citation STUDY root")?;
    let output = governed_output_path(&study_root, &request.output)?;

    let publication_reference = study_reference(
        &study_root,
        &request.publication_receipt_uri,
        "benchmark citation publication receipt",
    )?;
    let publication =
        load_indexed_receipt(&study_root, &indexed_reference(&publication_reference))?;
    let index: BenchmarkRunIndex =
        read_json(&repository_root.join(RUN_INDEX_PATH), "benchmark citation run index")?;
    validate_benchmark_run_index(&index).context("validate benchmark citation run index")?;
    let run = index
        .runs
        .iter()
        .find(|run| run.run_id == publication.run.run_id)
        .context("benchmark citation run is absent from the permanent index")?;
    validate_publication_protocol(run, &publication)?;

    let evidence_book_reference = study_reference(
        &study_root,
        &request.evidence_book_uri,
        "benchmark citation evidence book",
    )?;
    validate_book_admission(
        &study_root,
        &evidence_book_reference,
        &publication_reference,
        &publication,
        &run.citation_key,
    )?;
    let matrix_reference =
        study_reference(&study_root, &request.case_matrix_uri, "benchmark citation case matrix")?;
    if !publication.evidence.contains(&matrix_reference) {
        bail!("benchmark citation case matrix is not retained by the publication receipt");
    }
    validate_exact_replay(
        &study_root,
        &request.replay_receipt_uri,
        &evidence_book_reference,
        &publication_reference,
        &publication,
        &matrix_reference,
    )?;

    let source_root = study_root
        .join("data/evidence/source-checkouts")
        .join(&publication.repository_commit)
        .canonicalize()
        .context("canonicalize benchmark citation source checkout")?;
    if !source_root.starts_with(&study_root)
        || bijux_dna_containers::api::clean_repository_head_commit(&source_root)?
            != publication.repository_commit
    {
        bail!("benchmark citation source checkout differs from the publication commit");
    }
    let source_catalog_sha256 = bijux_dna_containers::api::catalog_identity_at_commit(
        &source_root,
        &publication.repository_commit,
    )?;
    if source_catalog_sha256 != publication.catalog_sha256 {
        bail!("benchmark citation catalog differs from the publication receipt");
    }
    let definition = load_source_definition(&source_root, run, &publication)?;
    let report: BenchmarkStageReport =
        read_study_json(&study_root, &publication.report.uri, "benchmark citation report")?;
    verify_study_reference(
        &study_root,
        &publication.report,
        "benchmark citation publication report",
    )?;
    validate_stage_report(&report).context("validate benchmark citation report")?;
    if report.run_id != publication.run.run_id
        || report.stage_id != publication.run.stage_id
        || report.catalog_sha256 != publication.catalog_sha256
    {
        bail!("benchmark citation report differs from the publication identity");
    }
    let catalog = bijux_dna_containers::api::load_container_catalog(&source_root)?;
    let tools = citation_tools(&publication, &report, &catalog)?;
    let matrix: bijux_dna_bench_model::BenchmarkCaseMatrixV2 =
        read_study_json(&study_root, &request.case_matrix_uri, "benchmark citation case matrix")?;
    validate_benchmark_case_matrix(&matrix).context("validate benchmark citation case matrix")?;
    validate_matrix_selection(&matrix, &publication)?;
    let assets = citation_assets(&matrix);
    let limitations = citation_limitations(&publication.limitations, &request.limitations);

    let citation = BenchmarkCitationRecord {
        schema_version: BENCHMARK_CITATION_RECORD_SCHEMA_V1.to_string(),
        citation_key: run.citation_key.clone(),
        run: publication.run.clone(),
        protocol_title: definition.title,
        protocol_version: definition.protocol_version,
        protocol_sha256: publication.run_definition_sha256.clone(),
        publication_id: publication.publication_id.clone(),
        selected_attempts: publication.selected_attempts.clone(),
        repository_commit: publication.repository_commit.clone(),
        tools,
        assets,
        report: publication.report.clone(),
        evidence_book: evidence_book_reference,
        publication_receipt: publication_reference,
        limitations,
        access_statement: request.access_statement.clone(),
    };
    validate_benchmark_citation_record(&citation).context("validate benchmark citation record")?;
    publish_identity(&output, &citation)?;
    Ok(citation)
}

fn validate_publication_protocol(
    run: &BenchmarkRunIndexEntry,
    publication: &BenchmarkPublicationReceipt,
) -> Result<()> {
    if publication.run.run_id != run.run_id
        || publication.run.stage_id != run.stage_id
        || publication.run_definition_sha256 != run.protocol_sha256
    {
        bail!("benchmark citation publication differs from the permanent protocol");
    }
    Ok(())
}

fn validate_book_admission(
    study_root: &Path,
    book_reference: &BenchmarkEvidenceReference,
    publication_reference: &BenchmarkEvidenceReference,
    publication: &BenchmarkPublicationReceipt,
    citation_key: &str,
) -> Result<()> {
    let book: BenchmarkEvidenceBook =
        read_study_json(study_root, &book_reference.uri, "benchmark citation evidence book")?;
    validate_benchmark_evidence_book(&book).context("validate benchmark citation evidence book")?;
    let entry = book
        .entries
        .iter()
        .find(|entry| entry.run == publication.run)
        .context("benchmark citation evidence book omits the publication run")?;
    if entry.publication_id != publication.publication_id
        || entry.citation_key != citation_key
        || entry.publication_receipt != *publication_reference
    {
        bail!("benchmark citation evidence book differs from the publication receipt");
    }
    Ok(())
}

fn validate_exact_replay(
    study_root: &Path,
    replay_uri: &str,
    book_reference: &BenchmarkEvidenceReference,
    publication_reference: &BenchmarkEvidenceReference,
    publication: &BenchmarkPublicationReceipt,
    matrix_reference: &BenchmarkEvidenceReference,
) -> Result<()> {
    let replay: BenchmarkReplayReceipt =
        read_study_json(study_root, replay_uri, "benchmark citation replay receipt")?;
    validate_benchmark_replay_receipt(&replay)
        .context("validate benchmark citation replay receipt")?;
    if replay.outcome != BenchmarkReplayOutcome::ExactMatch
        || replay.run != publication.run
        || replay.publication_id != publication.publication_id
        || replay.selected_attempts != publication.selected_attempts
        || replay.source_repository_commit != publication.repository_commit
        || replay.source_publication != *publication_reference
        || replay.expected.evidence_book_sha256 != book_reference.sha256
        || !replay.immutable_inputs.contains(matrix_reference)
    {
        bail!("benchmark citation replay differs from the admitted publication closure");
    }
    Ok(())
}

fn load_source_definition(
    source_root: &Path,
    run: &BenchmarkRunIndexEntry,
    publication: &BenchmarkPublicationReceipt,
) -> Result<BenchmarkRunDefinition> {
    let path = source_root
        .join("benchmarks/runs")
        .join(format!("{}-{}", run.run_id, run.slug))
        .join("definition.json");
    let definition: BenchmarkRunDefinition = read_json(&path, "benchmark citation definition")?;
    validate_benchmark_run_definition(&definition)
        .context("validate benchmark citation definition")?;
    if definition.identity != publication.run
        || benchmark_run_definition_sha256(&definition)? != publication.run_definition_sha256
    {
        bail!("benchmark citation definition differs from the publication protocol");
    }
    Ok(definition)
}

fn citation_tools(
    publication: &BenchmarkPublicationReceipt,
    report: &BenchmarkStageReport,
    catalog: &bijux_dna_containers::api::ContainerCatalog,
) -> Result<Vec<BenchmarkCitationToolIdentity>> {
    if report.tools.len() != publication.selected_attempts.len() {
        bail!("benchmark citation report tool closure differs from selected attempts");
    }
    publication
        .selected_attempts
        .iter()
        .map(|attempt| {
            let tool = catalog
                .tools
                .iter()
                .find(|tool| tool.id == attempt.tool_id)
                .context("benchmark citation catalog omits a selected tool")?;
            let observed = report
                .tools
                .iter()
                .find(|observed| observed.tool == attempt.tool_id)
                .context("benchmark citation report omits a selected tool")?;
            if observed.repository_commit != publication.repository_commit {
                bail!("benchmark citation tool report differs from the publication source");
            }
            Ok(BenchmarkCitationToolIdentity {
                tool_id: attempt.tool_id.clone(),
                tool_version: tool.version.clone(),
                image_sha256: observed.image.sha256.clone(),
            })
        })
        .collect()
}

fn validate_matrix_selection(
    matrix: &bijux_dna_bench_model::BenchmarkCaseMatrixV2,
    publication: &BenchmarkPublicationReceipt,
) -> Result<()> {
    let exact_cases = publication.selected_attempts.iter().all(|attempt| {
        attempt.case_id.as_deref().is_some_and(|case_id| {
            matrix.cases.iter().any(|case| {
                case.case_id == case_id
                    && case.run_id == attempt.run.run_id
                    && case.domain == attempt.run.domain
                    && case.stage_id == attempt.run.stage_id
                    && case.tool_id == attempt.tool_id
            })
        })
    });
    if matrix.repository_commit != publication.repository_commit
        || matrix.cases.len() != publication.case_ids.len()
        || !exact_cases
        || matrix.cases.iter().any(|case| {
            case.run_id != publication.run.run_id
                || case.domain != publication.run.domain
                || case.stage_id != publication.run.stage_id
                || !publication.case_ids.contains(&case.case_id)
        })
    {
        bail!("benchmark citation case matrix differs from the publication selection");
    }
    Ok(())
}

fn citation_assets(
    matrix: &bijux_dna_bench_model::BenchmarkCaseMatrixV2,
) -> Vec<BenchmarkCitationAssetIdentity> {
    citation_assets_from_snapshots(&matrix.data_snapshots)
}

fn citation_assets_from_snapshots(
    snapshots: &[BenchmarkDataSnapshotV1],
) -> Vec<BenchmarkCitationAssetIdentity> {
    let mut assets = snapshots
        .iter()
        .map(|snapshot| BenchmarkCitationAssetIdentity {
            role: match snapshot.role {
                BenchmarkDataSnapshotRoleV1::Corpus => "corpus",
                BenchmarkDataSnapshotRoleV1::Reference => "reference",
                BenchmarkDataSnapshotRoleV1::Database => "database",
            }
            .to_string(),
            asset_id: snapshot.catalog_id.clone(),
            sha256: snapshot.catalog_sha256.clone(),
        })
        .collect::<Vec<_>>();
    assets.sort_by(|left, right| (&left.role, &left.asset_id).cmp(&(&right.role, &right.asset_id)));
    assets
}

fn citation_limitations(source: &[String], declared: &[String]) -> Vec<String> {
    let mut limitations = source.iter().chain(declared).cloned().collect::<Vec<_>>();
    limitations.sort();
    limitations.dedup();
    limitations
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
        .context("validated benchmark citation URI lost its prefix")?;
    read_json(&study_root.join(relative), label)
}

fn publish_identity(output: &Path, citation: &BenchmarkCitationRecord) -> Result<()> {
    let mut expected =
        serde_json::to_vec_pretty(citation).context("serialize benchmark citation record")?;
    expected.push(b'\n');
    match fs::symlink_metadata(output) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark citation output is not a regular file");
            }
            if metadata.len() > MAX_CITATION_BYTES {
                bail!("existing benchmark citation exceeds the bounded size limit");
            }
            if fs::read(output)? != expected {
                bail!("refusing to replace a different benchmark citation record");
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(output, &expected)?;
            Ok(())
        }
        Err(error) => Err(error).context("inspect benchmark citation output"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citation_assets_are_role_and_identity_sorted() {
        let snapshots = vec![
            BenchmarkDataSnapshotV1 {
                catalog_id: "grch38-reference".to_string(),
                role: BenchmarkDataSnapshotRoleV1::Reference,
                catalog_sha256: "b".repeat(64),
                sample_ids: Vec::new(),
            },
            BenchmarkDataSnapshotV1 {
                catalog_id: "human-fastq-cohort".to_string(),
                role: BenchmarkDataSnapshotRoleV1::Corpus,
                catalog_sha256: "a".repeat(64),
                sample_ids: vec!["sample-1".to_string()],
            },
        ];
        let assets = citation_assets_from_snapshots(&snapshots);
        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].role, "corpus");
        assert_eq!(assets[0].asset_id, "human-fastq-cohort");
        assert_eq!(assets[1].role, "reference");
        assert_eq!(assets[1].asset_id, "grch38-reference");
    }

    #[test]
    fn citation_retains_source_limitations_without_duplicates() {
        let limitations = citation_limitations(
            &["The source publication covers one governed cohort.".to_string()],
            &[
                "The citation is limited to admitted evidence.".to_string(),
                "The source publication covers one governed cohort.".to_string(),
            ],
        );
        assert_eq!(limitations.len(), 2);
        assert_eq!(limitations[0], "The citation is limited to admitted evidence.");
        assert_eq!(limitations[1], "The source publication covers one governed cohort.");
    }
}
