use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_domain_fastq::{
    EstimateLibraryComplexityPrealignReportV1, PairedMode,
};
use serde::{Deserialize, Serialize};

const LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.estimate_library_complexity_prealign.local_smoke.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LocalEstimateLibraryComplexityPrealignSmokeStatus {
    ComplexityEstimated,
    InsufficientReads,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalEstimateLibraryComplexityPrealignSmokeCaseReport {
    sample_id: String,
    layout: PairedMode,
    input_r1: String,
    input_r2: Option<String>,
    reads_in: u64,
    estimated_unique_fraction: f64,
    estimated_duplicate_fraction: f64,
    kmer_size: Option<u32>,
    complexity_policy: String,
    estimate_method: String,
    complexity_status: LocalEstimateLibraryComplexityPrealignSmokeStatus,
    report_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalEstimateLibraryComplexityPrealignSmokeReport {
    schema_version: String,
    stage_id: String,
    case_count: u64,
    estimated_case_count: u64,
    insufficient_reads_case_count: u64,
    cases: Vec<LocalEstimateLibraryComplexityPrealignSmokeCaseReport>,
}

/// Materialize the governed local-smoke
/// `fastq.estimate_library_complexity_prealign` report bundle.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_estimate_library_complexity_prealign_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_fastq::stage_api::local_estimate_library_complexity_prealign_smoke_plans(
        &repo_root,
    )?;
    let output_root = repo_root.join("target/local-smoke/fastq.estimate_library_complexity_prealign");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let case_reports = cases
        .iter()
        .map(|case| materialize_local_estimate_library_complexity_prealign_smoke_case(&repo_root, case))
        .collect::<Result<Vec<_>>>()?;

    let summary = LocalEstimateLibraryComplexityPrealignSmokeReport {
        schema_version:
            LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "fastq.estimate_library_complexity_prealign".to_string(),
        case_count: case_reports.len() as u64,
        estimated_case_count: case_reports
            .iter()
            .filter(|case| {
                case.complexity_status
                    == LocalEstimateLibraryComplexityPrealignSmokeStatus::ComplexityEstimated
            })
            .count() as u64,
        insufficient_reads_case_count: case_reports
            .iter()
            .filter(|case| {
                case.complexity_status
                    == LocalEstimateLibraryComplexityPrealignSmokeStatus::InsufficientReads
            })
            .count() as u64,
        cases: case_reports,
    };

    let report_path = output_root.join("complexity.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(report_path)
}

fn materialize_local_estimate_library_complexity_prealign_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_fastq::LocalEstimateLibraryComplexityPrealignSmokeCasePlan,
) -> Result<LocalEstimateLibraryComplexityPrealignSmokeCaseReport> {
    let case_out_dir = resolve_plan_dir(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let report_json = case_out_dir.join("library_complexity_report.json");
    let r1 = repo_root.join(&case.r1);
    let r2 = case.r2.as_ref().map(|path| repo_root.join(path));
    let report = bijux_dna_domain_fastq::stages::estimate_library_complexity_prealign(
        &r1,
        r2.as_deref(),
        Some(case.kmer_size),
    )?;

    bijux_dna_infra::atomic_write_json(&report_json, &report)?;

    Ok(LocalEstimateLibraryComplexityPrealignSmokeCaseReport {
        sample_id: case.sample_id.clone(),
        layout: if case.r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        input_r1: case.r1.display().to_string(),
        input_r2: case.r2.as_ref().map(|path| path.display().to_string()),
        reads_in: report.reads_in,
        estimated_unique_fraction: report.estimated_unique_fraction,
        estimated_duplicate_fraction: report.estimated_duplicate_fraction,
        kmer_size: report.kmer_size,
        complexity_policy: report.complexity_policy.clone(),
        estimate_method: report.estimate_method.clone(),
        complexity_status: complexity_status(&report),
        report_json: path_relative_to_repo(repo_root, &report_json),
    })
}

fn complexity_status(
    report: &EstimateLibraryComplexityPrealignReportV1,
) -> LocalEstimateLibraryComplexityPrealignSmokeStatus {
    if report.reads_in == 0 {
        LocalEstimateLibraryComplexityPrealignSmokeStatus::InsufficientReads
    } else {
        LocalEstimateLibraryComplexityPrealignSmokeStatus::ComplexityEstimated
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
