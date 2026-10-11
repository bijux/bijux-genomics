//! Owner: bijux-dna-bench
//! Immutable publication receipts derived from admitted benchmark attempts.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    benchmark_run_definition_sha256, validate_benchmark_run_definition, validate_stage_report,
};
use bijux_dna_bench_model::{
    validate_benchmark_publication_receipt, validate_benchmark_run_index, BenchmarkAttemptIdentity,
    BenchmarkCampaignPlan, BenchmarkCaseMatrixV2, BenchmarkLifecycleStatus,
    BenchmarkPublicationReceipt, BenchmarkReportVerdict, BenchmarkRunDefinition, BenchmarkRunIndex,
    BenchmarkStageReport, BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1,
};

use super::campaign_lifecycle::verify_case_campaign_plan_source;
use super::evidence_book::{
    governed_output_path, read_json, study_reference, verify_study_reference,
};
use super::stage_report::is_selected_plan_attempt;
use crate::load_attempt_lifecycle;

const RUN_INDEX_PATH: &str = "benchmarks/runs/RUN_INDEX.json";
const CAMPAIGN_REGISTRY_PATH: &str = "benchmarks/contracts/stage-campaigns.toml";
const MAX_RECEIPT_BYTES: u64 = 32 * 1024 * 1024;

/// Inputs for one source-locked immutable benchmark publication receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkPublicationReceiptRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub benchmark_root: PathBuf,
    pub plan_uri: String,
    pub case_matrix_uri: String,
    pub report_uri: String,
    pub decision_uri: String,
    pub publication_id: String,
    pub published_at_utc: String,
    pub limitations: Vec<String>,
    pub output: PathBuf,
}

/// Issue one immutable receipt from a complete, passing case-scoped campaign.
///
/// The selected attempts come from the plan's lineage tips, not caller-provided identities. Every
/// selected lifecycle must be terminal-passing and retain the exact evidence digest used by the
/// passing stage report. The plan and case matrix are included in the evidence set so retry and
/// supersession lineage remains resolvable from the receipt.
///
/// # Errors
/// Returns an error when source authorities, attempt lineage, report evidence, STUDY references,
/// or immutable output publication are incomplete or inconsistent.
pub fn issue_benchmark_publication_receipt(
    request: &BenchmarkPublicationReceiptRequest,
) -> Result<BenchmarkPublicationReceipt> {
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize benchmark publication repository")?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&repository_root)
            .context("require a clean benchmark publication repository")?;
    let catalog_sha256 = bijux_dna_containers::api::catalog_identity_at_commit(
        &repository_root,
        &repository_commit,
    )?;
    let campaign_registry_sha256 =
        bijux_dna_infra::hash_file_sha256(&repository_root.join(CAMPAIGN_REGISTRY_PATH))?;
    let study_root = request
        .study_root
        .canonicalize()
        .context("canonicalize benchmark publication STUDY root")?;
    let benchmark_root = request
        .benchmark_root
        .canonicalize()
        .context("canonicalize benchmark publication attempt root")?;
    if !benchmark_root.starts_with(&study_root) {
        bail!("benchmark publication attempt root escapes governed STUDY");
    }
    let output = governed_output_path(&study_root, &request.output)?;

    let plan_reference =
        study_reference(&study_root, &request.plan_uri, "benchmark publication plan")?;
    let matrix_reference = study_reference(
        &study_root,
        &request.case_matrix_uri,
        "benchmark publication case matrix",
    )?;
    let plan: BenchmarkCampaignPlan =
        read_study_json(&study_root, &request.plan_uri, "benchmark publication plan")?;
    let matrix: BenchmarkCaseMatrixV2 = read_study_json(
        &study_root,
        &request.case_matrix_uri,
        "benchmark publication case matrix",
    )?;
    verify_case_campaign_plan_source(&plan, &matrix)?;
    if plan.repository_commit != repository_commit
        || plan.campaign_registry_sha256 != campaign_registry_sha256
    {
        bail!("benchmark publication plan differs from the exact source authority");
    }

    let selected_attempts = selected_passing_attempts(&benchmark_root, &plan)?;
    let run =
        selected_attempts.first().context("benchmark publication selects no attempts")?.run.clone();
    let case_ids = matrix_case_ids(&matrix, &run.run_id)?;
    let selected_case_ids = selected_attempts
        .iter()
        .map(|attempt| {
            attempt
                .case_id
                .clone()
                .context("case-scoped publication attempt omits its case identity")
        })
        .collect::<Result<BTreeSet<_>>>()?;
    if selected_attempts.len() != case_ids.len()
        || selected_case_ids != case_ids.iter().cloned().collect()
    {
        bail!("benchmark publication attempts do not close over the exact case matrix");
    }

    let report_reference =
        study_reference(&study_root, &request.report_uri, "benchmark publication report")?;
    let decision_reference =
        study_reference(&study_root, &request.decision_uri, "benchmark publication decision")?;
    let report: BenchmarkStageReport =
        read_study_json(&study_root, &request.report_uri, "benchmark publication report")?;
    validate_report_selection(
        &report,
        &run.run_id,
        &run.stage_id,
        &repository_commit,
        &catalog_sha256,
        &selected_attempts,
        &benchmark_root,
        &study_root,
    )?;

    let index: BenchmarkRunIndex =
        read_json(&repository_root.join(RUN_INDEX_PATH), "benchmark publication run index")?;
    validate_benchmark_run_index(&index).context("validate benchmark publication run index")?;
    let indexed_run = index
        .runs
        .iter()
        .find(|entry| entry.run_id == run.run_id)
        .context("benchmark publication run is absent from the permanent run index")?;
    if indexed_run.stage_id != run.stage_id {
        bail!("benchmark publication stage differs from the permanent run protocol");
    }
    let definition_path = repository_root
        .join("benchmarks/runs")
        .join(format!("{}-{}", indexed_run.run_id, indexed_run.slug))
        .join("definition.json");
    let definition: BenchmarkRunDefinition =
        read_json(&definition_path, "benchmark publication run definition")?;
    validate_benchmark_run_definition(&definition)
        .context("validate benchmark publication run definition")?;
    if definition.identity != run
        || benchmark_run_definition_sha256(&definition)? != indexed_run.protocol_sha256
    {
        bail!("benchmark publication run definition differs from the permanent run index");
    }

    let mut evidence = vec![plan_reference, matrix_reference];
    for identity in &selected_attempts {
        let lifecycle = load_attempt_lifecycle(&benchmark_root, identity)?;
        for reference in lifecycle.transitions.iter().flat_map(|transition| &transition.evidence) {
            verify_study_reference(
                &study_root,
                reference,
                "benchmark publication attempt evidence",
            )?;
            evidence.push(reference.clone());
        }
    }
    evidence.sort_by(|left, right| left.uri.cmp(&right.uri));
    evidence.dedup_by(|left, right| left.uri == right.uri);

    let receipt = BenchmarkPublicationReceipt {
        schema_version: BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1.to_string(),
        publication_id: request.publication_id.clone(),
        published_at_utc: request.published_at_utc.clone(),
        run,
        case_ids,
        selected_attempts,
        repository_commit,
        catalog_sha256,
        run_definition_sha256: indexed_run.protocol_sha256.clone(),
        campaign_registry_sha256,
        report: report_reference,
        decision: decision_reference,
        evidence,
        limitations: request.limitations.clone(),
    };
    validate_benchmark_publication_receipt(&receipt)
        .context("validate benchmark publication receipt")?;
    publish_identity(&output, &receipt)?;
    Ok(receipt)
}

fn selected_passing_attempts(
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
) -> Result<Vec<BenchmarkAttemptIdentity>> {
    let mut selected = plan
        .cycle
        .attempts
        .iter()
        .filter(|attempt| is_selected_plan_attempt(attempt))
        .map(|planned| {
            let lifecycle = load_attempt_lifecycle(benchmark_root, &planned.identity)?;
            if lifecycle.current_status() != Some(BenchmarkLifecycleStatus::Passed) {
                bail!("benchmark publication selects a non-passing lifecycle tip");
            }
            if lifecycle.supersedes_attempt_id != planned.supersedes_attempt_id {
                bail!("benchmark publication lifecycle supersession differs from its plan");
            }
            Ok(planned.identity.clone())
        })
        .collect::<Result<Vec<_>>>()?;
    selected.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    if selected.windows(2).any(|pair| pair[0].tool_id == pair[1].tool_id) {
        bail!("benchmark publication selects more than one passing attempt per tool");
    }
    Ok(selected)
}

fn matrix_case_ids(matrix: &BenchmarkCaseMatrixV2, run_id: &str) -> Result<Vec<String>> {
    matrix_case_ids_from_pairs(
        matrix.cases.iter().map(|case| (case.case_id.as_str(), case.run_id.as_str())),
        run_id,
    )
}

fn matrix_case_ids_from_pairs<'a>(
    cases: impl IntoIterator<Item = (&'a str, &'a str)>,
    run_id: &str,
) -> Result<Vec<String>> {
    let cases = cases.into_iter().collect::<Vec<_>>();
    if cases.iter().any(|(_, case_run_id)| *case_run_id != run_id) {
        bail!("benchmark publication case matrix mixes permanent runs");
    }
    let mut case_ids = cases.iter().map(|(case_id, _)| (*case_id).to_string()).collect::<Vec<_>>();
    case_ids.sort();
    case_ids.dedup();
    if case_ids.len() != cases.len() {
        bail!("benchmark publication case matrix contains duplicate case identities");
    }
    Ok(case_ids)
}

#[allow(clippy::too_many_arguments)]
fn validate_report_selection(
    report: &BenchmarkStageReport,
    run_id: &str,
    stage_id: &str,
    repository_commit: &str,
    catalog_sha256: &str,
    selected_attempts: &[BenchmarkAttemptIdentity],
    benchmark_root: &Path,
    study_root: &Path,
) -> Result<()> {
    validate_stage_report(report).context("validate publication stage report")?;
    if report.verdict != BenchmarkReportVerdict::Passed
        || report.run_id != run_id
        || report.stage_id != stage_id
        || report.catalog_sha256 != catalog_sha256
    {
        bail!("benchmark publication report differs from its source-locked run identity");
    }
    let reports_by_tool =
        report.tools.iter().map(|tool| (tool.tool.as_str(), tool)).collect::<BTreeMap<_, _>>();
    if reports_by_tool.len() != selected_attempts.len() {
        bail!("benchmark publication report tool closure differs from selected attempts");
    }
    for identity in selected_attempts {
        let tool = reports_by_tool
            .get(identity.tool_id.as_str())
            .with_context(|| format!("benchmark publication report omits {}", identity.tool_id))?;
        if tool.verdict != BenchmarkReportVerdict::Passed
            || tool.repository_commit != repository_commit
        {
            bail!("benchmark publication report contains non-passing or stale tool evidence");
        }
        let lifecycle = load_attempt_lifecycle(benchmark_root, identity)?;
        let retained = lifecycle
            .transitions
            .iter()
            .flat_map(|transition| &transition.evidence)
            .collect::<Vec<_>>();
        if retained.len() != 1 || retained[0].sha256 != tool.source_evidence_sha256 {
            bail!("benchmark publication report evidence differs from selected attempt evidence");
        }
        verify_study_reference(study_root, retained[0], "benchmark publication selected evidence")?;
    }
    Ok(())
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
        .context("validated STUDY reference lost its URI prefix")?;
    read_json(&study_root.join(relative), label)
}

fn publish_identity(output: &Path, receipt: &BenchmarkPublicationReceipt) -> Result<()> {
    let mut expected =
        serde_json::to_vec_pretty(receipt).context("serialize benchmark publication receipt")?;
    expected.push(b'\n');
    match fs::symlink_metadata(output) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark publication-receipt output is not a regular file");
            }
            if metadata.len() > MAX_RECEIPT_BYTES {
                bail!("existing benchmark publication receipt exceeds the bounded size limit");
            }
            if fs::read(output)? != expected {
                bail!("refusing to replace a different benchmark publication receipt");
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(output, &expected)?;
            Ok(())
        }
        Err(error) => Err(error).context("inspect benchmark publication-receipt output"),
    }
}

#[cfg(test)]
mod tests {
    use super::matrix_case_ids_from_pairs;

    #[test]
    fn publication_case_closure_rejects_mixed_runs() {
        assert!(matrix_case_ids_from_pairs(
            [("case-a", "run-2051"), ("case-b", "run-2052")],
            "run-2051",
        )
        .is_err());
    }

    #[test]
    fn publication_case_closure_is_sorted_and_unique() -> anyhow::Result<()> {
        assert_eq!(
            matrix_case_ids_from_pairs(
                [("case-b", "run-2051"), ("case-a", "run-2051")],
                "run-2051",
            )?,
            ["case-a", "case-b"]
        );
        assert!(matrix_case_ids_from_pairs(
            [("case-a", "run-2051"), ("case-a", "run-2051")],
            "run-2051",
        )
        .is_err());
        Ok(())
    }
}
