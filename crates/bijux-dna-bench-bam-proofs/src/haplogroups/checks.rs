use super::contracts::case_artifacts_present;
use super::contracts::case_parser_contract_ready;
use super::contracts::float_matches;
use super::contracts::insufficient_report_contract_ready;
use super::contracts::report_contract_ready;
use super::contracts::required_outputs_present;
use super::contracts::stage_metrics_ready;
use super::contracts::tool_specific_artifact_ready;
use super::contracts::ReportExpectation;
use super::declarations::EXPECTED_INSUFFICIENT_STATUS;
use super::declarations::EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION;
use super::declarations::EXPECTED_POPULATION_SCOPE;
use super::declarations::EXPECTED_READY_STATUS;
use super::declarations::EXPECTED_REFERENCE_BUILD;
use super::declarations::EXPECTED_REFERENCE_PANEL_ID;
use super::declarations::EXPECTED_SAMPLE_ID;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_SUMMARY_SCHEMA_VERSION;
use super::declarations::EXPECTED_TOOL_IDS;
use super::observations::LocalHaplogroupsSmokeCaseReport;
use anyhow::Result;
use std::path::Path;
pub(super) fn check_local_smoke_ready(
    repo_root: &Path,
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    insufficient_case: &LocalHaplogroupsSmokeCaseReport,
) -> bool {
    ready_case.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && insufficient_case.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && ready_case.stage_id == EXPECTED_STAGE_ID
        && insufficient_case.stage_id == EXPECTED_STAGE_ID
        && ready_case.tool_id == EXPECTED_TOOL_IDS[0]
        && insufficient_case.tool_id == EXPECTED_TOOL_IDS[0]
        && ready_case.sample_id == EXPECTED_SAMPLE_ID
        && insufficient_case.sample_id == EXPECTED_SAMPLE_ID
        && ready_case.reference_panel_id == EXPECTED_REFERENCE_PANEL_ID
        && insufficient_case.reference_panel_id == EXPECTED_REFERENCE_PANEL_ID
        && ready_case.reference_build == EXPECTED_REFERENCE_BUILD
        && insufficient_case.reference_build == EXPECTED_REFERENCE_BUILD
        && ready_case.population_scope == EXPECTED_POPULATION_SCOPE
        && insufficient_case.population_scope == EXPECTED_POPULATION_SCOPE
        && ready_case.expectation_matched
        && insufficient_case.expectation_matched
        && required_outputs_present(ready_case)
        && required_outputs_present(insufficient_case)
        && case_artifacts_present(ready_case)
        && case_artifacts_present(insufficient_case)
        && repo_root.join(&ready_case.reference_panel).is_file()
        && repo_root.join(&ready_case.reference_fasta).is_file()
        && repo_root.join(&ready_case.input_bam).is_file()
}

pub(super) fn check_ready_case_ready(
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    ready_report: &serde_json::Value,
) -> bool {
    report_contract_ready(
        ready_report,
        &ReportExpectation {
            ready: true,
            call: ready_case.haplogroup_call.as_deref(),
            confidence: ready_case.confidence,
            status: EXPECTED_READY_STATUS,
            minimum_coverage: ready_case.minimum_coverage,
            observed_mean_coverage: ready_case.observed_mean_coverage,
            markers_total: ready_case.markers_total,
            markers_supported: ready_case.markers_supported,
        },
    ) && ready_case.ready
        && ready_case.markers_supported == ready_case.markers_total
        && ready_case.haplogroup_call.as_deref() == Some("R1b1a")
}

pub(super) fn check_parser_contract_ready(
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    insufficient_case: &LocalHaplogroupsSmokeCaseReport,
) -> bool {
    case_parser_contract_ready(ready_case) && case_parser_contract_ready(insufficient_case)
}

pub(super) fn check_ready_summary_ready(
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    ready_summary: &bijux_dna_domain_bam::BamHaplogroupReadinessV1,
) -> bool {
    ready_summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && ready_summary.stage_id == EXPECTED_STAGE_ID
        && ready_summary.ready
        && ready_summary.reference_build.as_deref() == Some(EXPECTED_REFERENCE_BUILD)
        && float_matches(ready_summary.minimum_coverage, ready_case.minimum_coverage)
        && float_matches(ready_summary.observed_mean_coverage, ready_case.observed_mean_coverage)
        && ready_summary.contamination_estimate == Some(ready_case.contamination_estimate)
        && ready_summary.refusal_codes.is_empty()
}

pub(super) fn check_ready_stage_metrics_ready(
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    ready_stage_metrics: &serde_json::Value,
) -> bool {
    stage_metrics_ready(
        ready_stage_metrics,
        "ready",
        true,
        ready_case.minimum_coverage,
        ready_case.observed_mean_coverage,
        EXPECTED_READY_STATUS,
    )
}

pub(super) fn check_insufficient_summary_ready(
    insufficient_case: &LocalHaplogroupsSmokeCaseReport,
    insufficient_report: &serde_json::Value,
    insufficient_summary: &bijux_dna_domain_bam::BamHaplogroupReadinessV1,
) -> bool {
    insufficient_report_contract_ready(
        insufficient_report,
        &ReportExpectation {
            ready: false,
            call: None,
            confidence: 0.0,
            status: EXPECTED_INSUFFICIENT_STATUS,
            minimum_coverage: insufficient_case.minimum_coverage,
            observed_mean_coverage: insufficient_case.observed_mean_coverage,
            markers_total: insufficient_case.markers_total,
            markers_supported: insufficient_case.markers_supported,
        },
    ) && !insufficient_summary.ready
        && insufficient_summary.reference_build.as_deref() == Some(EXPECTED_REFERENCE_BUILD)
        && float_matches(insufficient_summary.minimum_coverage, insufficient_case.minimum_coverage)
        && float_matches(
            insufficient_summary.observed_mean_coverage,
            insufficient_case.observed_mean_coverage,
        )
        && insufficient_summary.contamination_estimate
            == Some(insufficient_case.contamination_estimate)
        && insufficient_summary.refusal_codes
            == vec!["coverage_below_haplogroup_minimum".to_string()]
}

pub(super) fn check_insufficient_stage_metrics_ready(
    insufficient_case: &LocalHaplogroupsSmokeCaseReport,
    insufficient_stage_metrics: &serde_json::Value,
) -> bool {
    stage_metrics_ready(
        insufficient_stage_metrics,
        "insufficient",
        false,
        insufficient_case.minimum_coverage,
        insufficient_case.observed_mean_coverage,
        EXPECTED_INSUFFICIENT_STATUS,
    )
}

pub(super) fn check_coverage_gate_ready(
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    insufficient_case: &LocalHaplogroupsSmokeCaseReport,
) -> bool {
    ready_case.minimum_coverage <= ready_case.observed_mean_coverage
        && insufficient_case.minimum_coverage > insufficient_case.observed_mean_coverage
}

pub(super) fn check_tool_specific_artifact_ready(
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    insufficient_case: &LocalHaplogroupsSmokeCaseReport,
    ready_tool_specific_artifact_path: &Path,
    insufficient_tool_specific_artifact_path: &Path,
) -> Result<bool> {
    tool_specific_artifact_ready(
        ready_tool_specific_artifact_path,
        insufficient_tool_specific_artifact_path,
        ready_case,
        insufficient_case,
    )
}
