use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_domain_fastq::{DetectDuplicatesPremergeReportV1, PairedMode};
use serde::{Deserialize, Serialize};

const LOCAL_DETECT_DUPLICATES_PREMERGE_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.detect_duplicates_premerge.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LocalDetectDuplicatesPremergeSmokeStatus {
    DuplicateSignalDetected,
    NoDuplicateSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalDetectDuplicatesPremergeSmokeCaseReport {
    sample_id: String,
    layout: PairedMode,
    input_r1: String,
    input_r2: Option<String>,
    reads_in: u64,
    duplicate_signal_reads: u64,
    duplicate_signal_fraction: f64,
    inspected_read_pair_count: Option<u64>,
    duplicate_detection_policy: String,
    measurement_scope: String,
    duplicate_status: LocalDetectDuplicatesPremergeSmokeStatus,
    report_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalDetectDuplicatesPremergeSmokeReport {
    schema_version: String,
    stage_id: String,
    case_count: u64,
    duplicate_signal_case_count: u64,
    no_duplicate_signal_case_count: u64,
    cases: Vec<LocalDetectDuplicatesPremergeSmokeCaseReport>,
}

/// Materialize the governed local-smoke `fastq.detect_duplicates_premerge` report bundle.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_detect_duplicates_premerge_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_fastq::stage_api::local_detect_duplicates_premerge_smoke_plans(
        &repo_root,
    )?;
    let output_root = repo_root.join("target/local-smoke/fastq.detect_duplicates_premerge");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let case_reports = cases
        .iter()
        .map(|case| materialize_local_detect_duplicates_premerge_smoke_case(&repo_root, case))
        .collect::<Result<Vec<_>>>()?;

    let summary = LocalDetectDuplicatesPremergeSmokeReport {
        schema_version: LOCAL_DETECT_DUPLICATES_PREMERGE_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "fastq.detect_duplicates_premerge".to_string(),
        case_count: case_reports.len() as u64,
        duplicate_signal_case_count: case_reports
            .iter()
            .filter(|case| {
                case.duplicate_status
                    == LocalDetectDuplicatesPremergeSmokeStatus::DuplicateSignalDetected
            })
            .count() as u64,
        no_duplicate_signal_case_count: case_reports
            .iter()
            .filter(|case| {
                case.duplicate_status == LocalDetectDuplicatesPremergeSmokeStatus::NoDuplicateSignal
            })
            .count() as u64,
        cases: case_reports,
    };

    let report_path = output_root.join("duplicates.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(report_path)
}

fn materialize_local_detect_duplicates_premerge_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalDetectDuplicatesPremergeSmokeCasePlan,
) -> Result<LocalDetectDuplicatesPremergeSmokeCaseReport> {
    let case_out_dir = resolve_plan_dir(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let report_json = case_out_dir.join("duplicate_signal_report.json");
    let r1 = repo_root.join(&case.r1);
    let r2 = case.r2.as_ref().map(|path| repo_root.join(path));
    let report = bijux_dna_domain_fastq::stages::detect_duplicates_premerge(&r1, r2.as_deref())?;

    bijux_dna_infra::atomic_write_json(&report_json, &report)?;

    Ok(LocalDetectDuplicatesPremergeSmokeCaseReport {
        sample_id: case.sample_id.clone(),
        layout: if case.r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        input_r1: case.r1.display().to_string(),
        input_r2: case.r2.as_ref().map(|path| path.display().to_string()),
        reads_in: report.reads_in,
        duplicate_signal_reads: report.duplicate_signal_reads,
        duplicate_signal_fraction: report.duplicate_signal_fraction,
        inspected_read_pair_count: report.compared_read_pairs,
        duplicate_detection_policy: report.duplicate_detection_policy.clone(),
        measurement_scope: report.measurement_scope.clone(),
        duplicate_status: duplicate_status(&report),
        report_json: path_relative_to_repo(repo_root, &report_json),
    })
}

fn duplicate_status(
    report: &DetectDuplicatesPremergeReportV1,
) -> LocalDetectDuplicatesPremergeSmokeStatus {
    if report.duplicate_signal_reads > 0 {
        LocalDetectDuplicatesPremergeSmokeStatus::DuplicateSignalDetected
    } else {
        LocalDetectDuplicatesPremergeSmokeStatus::NoDuplicateSignal
    }
}

fn resolve_plan_dir(repo_root: &Path, out_dir: &Path) -> PathBuf {
    if out_dir.is_absolute() {
        out_dir.to_path_buf()
    } else {
        repo_root.join(out_dir)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}
