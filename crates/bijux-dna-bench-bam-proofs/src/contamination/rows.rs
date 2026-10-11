use super::checks::check_insufficiency_behavior_ready;
use super::checks::check_insufficient_stage_metrics_ready;
use super::checks::check_insufficient_summary_ready;
use super::checks::check_local_smoke_ready;
use super::checks::check_parser_contract_ready;
use super::checks::check_ready_case_ready;
use super::checks::check_ready_stage_metrics_ready;
use super::checks::check_ready_summary_ready;
use super::explanation::explain;
use super::explanation::SurfaceChecks;
use super::models::BamContaminationCompleteRow;
use super::observations::LocalContaminationSmokeCaseReport;
use super::observations::LocalContaminationSmokeReport;
use super::projection::project;
use super::projection::RowProjection;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::readiness::contamination_sex_haplogroups::BamContaminationSexHaplogroupsReadyRow;
use std::path::Path;
pub(super) fn build_bam_contamination_complete_row(
    repo_root: &Path,
    smoke_path: &Path,
    smoke_report: &LocalContaminationSmokeReport,
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    ready_case: &LocalContaminationSmokeCaseReport,
    insufficient_case: &LocalContaminationSmokeCaseReport,
) -> Result<BamContaminationCompleteRow> {
    let ready_report_path = repo_root.join(&ready_case.contamination_report);
    let ready_summary_path = repo_root.join(&ready_case.contamination_summary);
    let ready_stage_metrics_path = repo_root.join(&ready_case.stage_metrics);
    let insufficient_summary_path = repo_root.join(&insufficient_case.contamination_summary);
    let insufficient_stage_metrics_path = repo_root.join(&insufficient_case.stage_metrics);

    let ready_raw_metrics =
        bijux_dna_domain_bam::metrics::parse_contamination_json(&ready_report_path)
            .with_context(|| format!("parse {}", ready_report_path.display()))?;
    let ready_summary: bijux_dna_domain_bam::BamContaminationEvidenceV1 =
        crate::input::read_json(&ready_summary_path)?;
    let insufficient_summary: bijux_dna_domain_bam::BamContaminationEvidenceV1 =
        crate::input::read_json(&insufficient_summary_path)?;
    let ready_stage_metrics: serde_json::Value =
        crate::input::read_json(&ready_stage_metrics_path)?;
    let insufficient_stage_metrics: serde_json::Value =
        crate::input::read_json(&insufficient_stage_metrics_path)?;

    let local_smoke_ready = check_local_smoke_ready(smoke_report, ready_case, insufficient_case);
    let ready_case_ready = check_ready_case_ready(readiness_row, ready_case, &ready_raw_metrics);
    let parser_contract_ready = check_parser_contract_ready(ready_case);
    let ready_summary_ready = check_ready_summary_ready(readiness_row, &ready_summary);
    let ready_stage_metrics_ready = check_ready_stage_metrics_ready(&ready_stage_metrics);
    let insufficient_summary_ready =
        check_insufficient_summary_ready(readiness_row, &insufficient_summary);
    let insufficient_stage_metrics_ready =
        check_insufficient_stage_metrics_ready(&insufficient_stage_metrics);
    let insufficiency_behavior_ready = check_insufficiency_behavior_ready(
        insufficient_case,
        insufficient_summary_ready,
        insufficient_stage_metrics_ready,
    );

    let checks = SurfaceChecks {
        local_smoke_ready,
        ready_case_ready,
        parser_contract_ready,
        ready_summary_ready,
        ready_stage_metrics_ready,
        insufficient_summary_ready,
        insufficient_stage_metrics_ready,
        insufficiency_behavior_ready,
    };
    let (missing_surfaces, coverage_status, reason) = explain(readiness_row, checks);

    project(RowProjection {
        ready_raw_metrics,
        repo_root,
        smoke_path,
        readiness_row,
        ready_case,
        insufficient_case,
        ready_stage_metrics_path,
        insufficient_stage_metrics_path,
        ready_summary,
        insufficient_summary,
        ready_stage_metrics,
        insufficient_stage_metrics,
        local_smoke_ready,
        ready_case_ready,
        parser_contract_ready,
        ready_summary_ready,
        ready_stage_metrics_ready,
        insufficient_summary_ready,
        insufficient_stage_metrics_ready,
        insufficiency_behavior_ready,
        missing_surfaces,
        coverage_status,
        reason,
    })
}
