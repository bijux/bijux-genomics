use super::contracts::case_artifacts_present;
use super::contracts::float_matches;
use super::contracts::insufficient_report_contract_ready;
use super::contracts::ready_report_contract_ready;
use super::contracts::required_outputs_present;
use super::contracts::sex_call_name;
use super::contracts::stage_metrics_ready;
use super::contracts::tool_specific_artifact_ready;
use super::declarations::EXPECTED_INSUFFICIENCY_REASON;
use super::declarations::EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_SUMMARY_SCHEMA_VERSION;
use super::observations::LocalSexToolSmokeCaseReport;
use super::observations::LocalSexToolSmokeReport;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::readiness::contamination_sex_haplogroups::BamContaminationSexHaplogroupsReadyRow;
use std::path::Path;
pub(super) fn check_local_smoke_ready(
    smoke_report: &LocalSexToolSmokeReport,
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    ready_case: &LocalSexToolSmokeCaseReport,
    insufficient_case: &LocalSexToolSmokeCaseReport,
) -> bool {
    ready_case.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && insufficient_case.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && ready_case.stage_id == EXPECTED_STAGE_ID
        && insufficient_case.stage_id == EXPECTED_STAGE_ID
        && ready_case.tool_id == readiness_row.tool_id
        && insufficient_case.tool_id == readiness_row.tool_id
        && ready_case.expectation_matched
        && insufficient_case.expectation_matched
        && ready_case.sample_id == smoke_report.ready_sample_id
        && insufficient_case.sample_id == smoke_report.insufficient_sample_id
        && ready_case.method == readiness_row.tool_id
        && insufficient_case.method == readiness_row.tool_id
        && required_outputs_present(ready_case)
        && required_outputs_present(insufficient_case)
        && case_artifacts_present(ready_case)
        && case_artifacts_present(insufficient_case)
}

pub(super) fn check_ready_case_ready(
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    ready_case: &LocalSexToolSmokeCaseReport,
    ready_report: &serde_json::Value,
    ready_summary: &bijux_dna_domain_bam::BamSexSummaryV1,
) -> bool {
    ready_report_contract_ready(ready_report, &readiness_row.tool_id)
        && ready_summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && ready_summary.stage_id == EXPECTED_STAGE_ID
        && ready_summary.method == readiness_row.tool_id
        && ready_summary.chromosome_system == ready_case.chromosome_system
        && ready_summary.minimum_y_sites == ready_case.minimum_y_sites
        && float_matches(ready_summary.x_coverage, ready_case.x_coverage)
        && float_matches(ready_summary.y_coverage, ready_case.y_coverage)
        && float_matches(ready_summary.autosomal_coverage, ready_case.autosomal_coverage)
        && ready_summary.x_to_y_ratio == ready_case.x_to_y_ratio
        && ready_summary.call == ready_case.call
        && float_matches(ready_summary.confidence, ready_case.confidence)
        && ready_summary.status == ready_case.status
        && ready_summary.insufficiency_reason.is_none()
        && ready_summary.x_coverage > 0.0
        && ready_summary.y_coverage > 0.0
        && ready_summary.autosomal_coverage > 0.0
        && ready_summary.x_covered_sites > 0
        && ready_summary.y_covered_sites >= u64::from(ready_summary.minimum_y_sites.unwrap_or(0))
        && ready_summary.x_to_y_ratio.is_some()
}

pub(super) fn check_parser_contract_ready(
    ready_case: &LocalSexToolSmokeCaseReport,
    insufficient_case: &LocalSexToolSmokeCaseReport,
) -> bool {
    ready_case.artifact_paths.iter().any(|path| path == &ready_case.sex_report)
        && ready_case.artifact_paths.iter().any(|path| path == &ready_case.sex_summary)
        && ready_case.artifact_paths.iter().any(|path| path == &ready_case.stage_metrics)
        && insufficient_case.artifact_paths.iter().any(|path| path == &insufficient_case.sex_report)
        && insufficient_case
            .artifact_paths
            .iter()
            .any(|path| path == &insufficient_case.sex_summary)
        && insufficient_case
            .artifact_paths
            .iter()
            .any(|path| path == &insufficient_case.stage_metrics)
}

pub(super) fn check_ready_summary_ready(ready_case_ready: bool) -> bool {
    ready_case_ready
}

pub(super) fn check_ready_stage_metrics_ready(
    ready_case: &LocalSexToolSmokeCaseReport,
    ready_stage_metrics: &serde_json::Value,
) -> bool {
    stage_metrics_ready(
        ready_stage_metrics,
        "ready",
        None,
        sex_call_name(ready_case.call),
        ready_case.confidence,
        &ready_case.status,
    )
}

pub(super) fn check_insufficient_summary_ready(
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    insufficient_case: &LocalSexToolSmokeCaseReport,
    insufficient_report: &serde_json::Value,
    insufficient_summary: &bijux_dna_domain_bam::BamSexSummaryV1,
) -> bool {
    insufficient_report_contract_ready(
        insufficient_report,
        &readiness_row.tool_id,
        EXPECTED_INSUFFICIENCY_REASON,
    ) && insufficient_summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && insufficient_summary.stage_id == EXPECTED_STAGE_ID
        && insufficient_summary.method == readiness_row.tool_id
        && insufficient_summary.call
            == bijux_dna_domain_bam::metrics::SexConfidenceClass::Insufficient
        && float_matches(insufficient_summary.confidence, 0.0)
        && insufficient_summary.status == EXPECTED_INSUFFICIENCY_REASON
        && insufficient_summary.insufficiency_reason.as_deref()
            == Some(EXPECTED_INSUFFICIENCY_REASON)
        && float_matches(insufficient_summary.x_coverage, 0.0)
        && float_matches(insufficient_summary.autosomal_coverage, 0.0)
        && float_matches(insufficient_summary.y_coverage, insufficient_case.y_coverage)
        && insufficient_summary.x_covered_sites == 0
        && insufficient_summary.y_covered_sites > 0
}

pub(super) fn check_insufficient_stage_metrics_ready(
    insufficient_stage_metrics: &serde_json::Value,
) -> bool {
    stage_metrics_ready(
        insufficient_stage_metrics,
        "insufficient",
        Some(EXPECTED_INSUFFICIENCY_REASON),
        "insufficient",
        0.0,
        EXPECTED_INSUFFICIENCY_REASON,
    )
}

pub(super) fn check_insufficiency_behavior_ready(
    insufficient_case: &LocalSexToolSmokeCaseReport,
    insufficient_summary_ready: bool,
    insufficient_stage_metrics_ready: bool,
) -> bool {
    insufficient_summary_ready
        && insufficient_stage_metrics_ready
        && insufficient_case.insufficiency_reason.as_deref() == Some(EXPECTED_INSUFFICIENCY_REASON)
        && insufficient_case.call == bijux_dna_domain_bam::metrics::SexConfidenceClass::Insufficient
        && float_matches(insufficient_case.confidence, 0.0)
        && insufficient_case.status == EXPECTED_INSUFFICIENCY_REASON
}

pub(super) fn check_tool_specific_artifact_ready(
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    ready_case: &LocalSexToolSmokeCaseReport,
    insufficient_case: &LocalSexToolSmokeCaseReport,
    ready_tool_specific_artifact_path: &Path,
    insufficient_tool_specific_artifact_path: &Path,
) -> Result<bool> {
    tool_specific_artifact_ready(
        &readiness_row.tool_id,
        ready_tool_specific_artifact_path,
        insufficient_tool_specific_artifact_path,
        ready_case,
        insufficient_case,
    )
}
