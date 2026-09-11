use super::contracts::float_matches;
use super::contracts::required_outputs_present;
use super::contracts::required_stage_metrics_ready;
use super::contracts::tool_specific_output_path;
use super::declarations::EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_SUMMARY_SCHEMA_VERSION;
use super::observations::LocalContaminationSmokeCaseReport;
use super::observations::LocalContaminationSmokeReport;
use bijux_dna_bench_bam_evidence::readiness::contamination_sex_haplogroups::BamContaminationSexHaplogroupsReadyRow;
pub(super) fn check_local_smoke_ready(
    smoke_report: &LocalContaminationSmokeReport,
    ready_case: &LocalContaminationSmokeCaseReport,
    insufficient_case: &LocalContaminationSmokeCaseReport,
) -> bool {
    ready_case.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && insufficient_case.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && ready_case.stage_id == EXPECTED_STAGE_ID
        && insufficient_case.stage_id == EXPECTED_STAGE_ID
        && ready_case.expectation_matched
        && insufficient_case.expectation_matched
        && ready_case.sample_id == smoke_report.sample_id
        && insufficient_case.sample_id == smoke_report.sample_id
        && required_outputs_present(ready_case)
        && required_outputs_present(insufficient_case)
        && tool_specific_output_path(ready_case).is_some()
        && tool_specific_output_path(insufficient_case).is_some()
        && !ready_case.artifact_paths.is_empty()
        && !insufficient_case.artifact_paths.is_empty()
}

pub(super) fn check_ready_case_ready(
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    ready_case: &LocalContaminationSmokeCaseReport,
    ready_raw_metrics: &bijux_dna_domain_bam::metrics::ContaminationMetricsV1,
) -> bool {
    ready_case.prerequisites_passed
        && ready_raw_metrics.method == readiness_row.tool_id
        && float_matches(ready_raw_metrics.estimate, 0.02)
        && float_matches(ready_raw_metrics.ci_low, 0.01)
        && float_matches(ready_raw_metrics.ci_high, 0.03)
}

pub(super) fn check_parser_contract_ready(ready_case: &LocalContaminationSmokeCaseReport) -> bool {
    ready_case.artifact_paths.iter().any(|path| path == &ready_case.contamination_report)
        && ready_case.artifact_paths.iter().any(|path| path == &ready_case.contamination_summary)
}

pub(super) fn check_ready_summary_ready(
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    ready_summary: &bijux_dna_domain_bam::BamContaminationEvidenceV1,
) -> bool {
    ready_summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && ready_summary.stage_id == EXPECTED_STAGE_ID
        && ready_summary.tool == readiness_row.tool_id
        && ready_summary.prerequisites_passed
        && ready_summary.estimate.is_some_and(|value| float_matches(value, 0.02))
        && ready_summary.ci_low.is_some_and(|value| float_matches(value, 0.01))
        && ready_summary.ci_high.is_some_and(|value| float_matches(value, 0.03))
}

pub(super) fn check_ready_stage_metrics_ready(ready_stage_metrics: &serde_json::Value) -> bool {
    required_stage_metrics_ready(ready_stage_metrics, "ready", true, true)
}

pub(super) fn check_insufficient_summary_ready(
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    insufficient_summary: &bijux_dna_domain_bam::BamContaminationEvidenceV1,
) -> bool {
    insufficient_summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && insufficient_summary.stage_id == EXPECTED_STAGE_ID
        && insufficient_summary.tool == readiness_row.tool_id
        && !insufficient_summary.prerequisites_passed
        && insufficient_summary.estimate.is_none()
        && insufficient_summary.ci_low.is_none()
        && insufficient_summary.ci_high.is_none()
        && !insufficient_summary.refusal_codes.is_empty()
}

pub(super) fn check_insufficient_stage_metrics_ready(
    insufficient_stage_metrics: &serde_json::Value,
) -> bool {
    required_stage_metrics_ready(insufficient_stage_metrics, "insufficient", false, false)
}

pub(super) fn check_insufficiency_behavior_ready(
    insufficient_case: &LocalContaminationSmokeCaseReport,
    insufficient_summary_ready: bool,
    insufficient_stage_metrics_ready: bool,
) -> bool {
    insufficient_summary_ready
        && insufficient_stage_metrics_ready
        && !insufficient_case.refusal_codes.is_empty()
        && !insufficient_case.prerequisites_passed
}
