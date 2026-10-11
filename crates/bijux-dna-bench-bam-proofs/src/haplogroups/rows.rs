use super::checks::check_coverage_gate_ready;
use super::checks::check_insufficient_stage_metrics_ready;
use super::checks::check_insufficient_summary_ready;
use super::checks::check_local_smoke_ready;
use super::checks::check_parser_contract_ready;
use super::checks::check_ready_case_ready;
use super::checks::check_ready_stage_metrics_ready;
use super::checks::check_ready_summary_ready;
use super::checks::check_tool_specific_artifact_ready;
use super::explanation::explain;
use super::explanation::SurfaceChecks;
use super::models::BamHaplogroupsCompleteRow;
use super::observations::LocalHaplogroupsSmokeCaseReport;
use super::observations::LocalHaplogroupsSmokeReport;
use super::projection::project;
use super::projection::RowProjection;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::readiness::contamination_sex_haplogroups::BamContaminationSexHaplogroupsReadyRow;
use std::path::Path;
pub(super) fn build_bam_haplogroups_complete_row(
    repo_root: &Path,
    smoke_path: &Path,
    smoke_report: &LocalHaplogroupsSmokeReport,
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    insufficient_case: &LocalHaplogroupsSmokeCaseReport,
) -> Result<BamHaplogroupsCompleteRow> {
    let ready_report_path = repo_root.join(&ready_case.haplogroups_report);
    let ready_summary_path = repo_root.join(&ready_case.haplogroups_summary);
    let ready_stage_metrics_path = repo_root.join(&ready_case.stage_metrics);
    let ready_tool_specific_artifact_path = repo_root.join(&ready_case.haplogroup_report);
    let insufficient_report_path = repo_root.join(&insufficient_case.haplogroups_report);
    let insufficient_summary_path = repo_root.join(&insufficient_case.haplogroups_summary);
    let insufficient_stage_metrics_path = repo_root.join(&insufficient_case.stage_metrics);
    let insufficient_tool_specific_artifact_path =
        repo_root.join(&insufficient_case.haplogroup_report);

    let ready_report: serde_json::Value = crate::input::read_json(&ready_report_path)?;
    let ready_summary: bijux_dna_domain_bam::BamHaplogroupReadinessV1 =
        crate::input::read_json(&ready_summary_path)?;
    let ready_stage_metrics: serde_json::Value =
        crate::input::read_json(&ready_stage_metrics_path)?;
    let insufficient_report: serde_json::Value =
        crate::input::read_json(&insufficient_report_path)?;
    let insufficient_summary: bijux_dna_domain_bam::BamHaplogroupReadinessV1 =
        crate::input::read_json(&insufficient_summary_path)?;
    let insufficient_stage_metrics: serde_json::Value =
        crate::input::read_json(&insufficient_stage_metrics_path)?;

    let local_smoke_ready = check_local_smoke_ready(repo_root, ready_case, insufficient_case);
    let ready_case_ready = check_ready_case_ready(ready_case, &ready_report);
    let parser_contract_ready = check_parser_contract_ready(ready_case, insufficient_case);
    let ready_summary_ready = check_ready_summary_ready(ready_case, &ready_summary);
    let ready_stage_metrics_ready =
        check_ready_stage_metrics_ready(ready_case, &ready_stage_metrics);
    let insufficient_summary_ready = check_insufficient_summary_ready(
        insufficient_case,
        &insufficient_report,
        &insufficient_summary,
    );
    let insufficient_stage_metrics_ready =
        check_insufficient_stage_metrics_ready(insufficient_case, &insufficient_stage_metrics);
    let coverage_gate_ready = check_coverage_gate_ready(ready_case, insufficient_case);
    let tool_specific_artifact_ready = check_tool_specific_artifact_ready(
        ready_case,
        insufficient_case,
        &ready_tool_specific_artifact_path,
        &insufficient_tool_specific_artifact_path,
    )?;

    let checks = SurfaceChecks {
        local_smoke_ready,
        ready_case_ready,
        parser_contract_ready,
        ready_summary_ready,
        ready_stage_metrics_ready,
        insufficient_summary_ready,
        insufficient_stage_metrics_ready,
        coverage_gate_ready,
        tool_specific_artifact_ready,
    };
    let (missing_surfaces, coverage_status, reason) = explain(readiness_row, &checks);

    project(RowProjection {
        repo_root,
        smoke_path,
        smoke_report,
        readiness_row,
        ready_case,
        insufficient_case,
        ready_stage_metrics_path,
        insufficient_stage_metrics_path,
        ready_summary,
        ready_stage_metrics,
        insufficient_summary,
        insufficient_stage_metrics,
        local_smoke_ready,
        ready_case_ready,
        parser_contract_ready,
        ready_summary_ready,
        ready_stage_metrics_ready,
        insufficient_summary_ready,
        insufficient_stage_metrics_ready,
        coverage_gate_ready,
        tool_specific_artifact_ready,
        missing_surfaces,
        coverage_status,
        reason,
    })
}
