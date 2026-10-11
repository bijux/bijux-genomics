use super::contracts::approx_eq;
use super::contracts::sorted_strings;
use super::contracts::sorted_strings_from_slice;
use super::declarations::EXPECTED_BAM_PIPELINE_ID;
use super::declarations::EXPECTED_INSUFFICIENCY_REASON;
use super::declarations::EXPECTED_INSUFFICIENT_CASE_OBSERVED_MAX_OVERLAP_SNPS;
use super::declarations::EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT;
use super::declarations::EXPECTED_INSUFFICIENT_CASE_SAMPLE_ID;
use super::declarations::EXPECTED_INSUFFICIENT_STATUS;
use super::declarations::EXPECTED_METHOD;
use super::declarations::EXPECTED_PAIRWISE_TABLE_ROW;
use super::declarations::EXPECTED_PARSER_SCHEMA_VERSION;
use super::declarations::EXPECTED_POPULATION_SCOPE;
use super::declarations::EXPECTED_READY_CASE_KINSHIP_COEFFICIENT;
use super::declarations::EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS;
use super::declarations::EXPECTED_READY_CASE_OVERLAP_SNPS;
use super::declarations::EXPECTED_READY_CASE_PAIR_COUNT;
use super::declarations::EXPECTED_READY_CASE_RELATIONSHIP_LABEL;
use super::declarations::EXPECTED_READY_CASE_SAMPLE_A;
use super::declarations::EXPECTED_READY_CASE_SAMPLE_B;
use super::declarations::EXPECTED_READY_CASE_SAMPLE_ID;
use super::declarations::EXPECTED_READY_STATUS;
use super::declarations::EXPECTED_REFERENCE_BUILD;
use super::declarations::EXPECTED_REFERENCE_PANEL;
use super::declarations::EXPECTED_REPORT_SCHEMA_VERSION;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_STAGE_METRICS_SCHEMA_VERSION;
use super::declarations::EXPECTED_SUMMARY_SCHEMA_VERSION;
use super::declarations::EXPECTED_VCF_PIPELINE_ID;
use super::declarations::REQUIRED_BAM_PIPELINE_DEPENDENCIES;
use super::declarations::REQUIRED_BAM_PIPELINE_EXTERNAL_INPUTS;
use super::declarations::REQUIRED_BAM_PIPELINE_OUTPUTS;
use super::declarations::REQUIRED_BAM_PIPELINE_UPSTREAM_INPUTS;
use super::input::optional_string;
use super::input::required_bool;
use super::input::required_pair_array_len;
use super::input::required_pair_f64;
use super::input::required_pair_field;
use super::input::required_pair_u64;
use super::input::required_string;
use super::input::required_u64;
use super::observations::LocalKinshipSmokeCaseReport;
use super::observations::LocalKinshipSmokeReport;
use crate::downstream::observations::ParserObservation;
use crate::downstream::observations::PipelineNode;
use crate::downstream::observations::PipelineObservation;
use anyhow::Result;
pub(super) fn check_local_smoke_ready(
    smoke_report: &LocalKinshipSmokeReport,
    ready_case: &LocalKinshipSmokeCaseReport,
    insufficient_case: &LocalKinshipSmokeCaseReport,
) -> bool {
    smoke_report.all_cases_matched
        && ready_case.expectation_matched
        && ready_case.sample_id == EXPECTED_READY_CASE_SAMPLE_ID
        && ready_case.method == EXPECTED_METHOD
        && ready_case.reference_panel == EXPECTED_REFERENCE_PANEL
        && ready_case.reference_build == EXPECTED_REFERENCE_BUILD
        && ready_case.population_scope == EXPECTED_POPULATION_SCOPE
        && ready_case.observed_max_overlap_snps == EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS
        && ready_case.pair_count == EXPECTED_READY_CASE_PAIR_COUNT
        && ready_case.status == EXPECTED_READY_STATUS
        && ready_case.insufficiency_reason.is_none()
        && ready_case.requires_cohort_context
        && ready_case.min_overlap_snps == EXPECTED_READY_CASE_OVERLAP_SNPS
        && insufficient_case.expectation_matched
        && insufficient_case.sample_id == EXPECTED_INSUFFICIENT_CASE_SAMPLE_ID
        && insufficient_case.method == EXPECTED_METHOD
        && insufficient_case.reference_panel == EXPECTED_REFERENCE_PANEL
        && insufficient_case.reference_build == EXPECTED_REFERENCE_BUILD
        && insufficient_case.population_scope == EXPECTED_POPULATION_SCOPE
        && insufficient_case.observed_max_overlap_snps
            == EXPECTED_INSUFFICIENT_CASE_OBSERVED_MAX_OVERLAP_SNPS
        && insufficient_case.pair_count == EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT
        && insufficient_case.status == EXPECTED_INSUFFICIENT_STATUS
        && insufficient_case.insufficiency_reason.as_deref() == Some(EXPECTED_INSUFFICIENCY_REASON)
        && insufficient_case.requires_cohort_context
        && insufficient_case.min_overlap_snps == 5
        && insufficient_case.pairwise_results.is_empty()
}

pub(super) fn check_ready_case_report_ready(ready_report: &serde_json::Value) -> Result<bool> {
    Ok(required_string(ready_report, "schema_version")? == EXPECTED_REPORT_SCHEMA_VERSION
        && required_string(ready_report, "status")? == EXPECTED_READY_STATUS
        && required_u64(ready_report, "pair_count")? == EXPECTED_READY_CASE_PAIR_COUNT
        && required_u64(ready_report, "observed_max_overlap_snps")?
            == EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS
        && required_pair_field(ready_report, 0, "sample_a")? == EXPECTED_READY_CASE_SAMPLE_A
        && required_pair_field(ready_report, 0, "sample_b")? == EXPECTED_READY_CASE_SAMPLE_B
        && required_pair_u64(ready_report, 0, "overlap_snps")? == EXPECTED_READY_CASE_OVERLAP_SNPS
        && approx_eq(
            required_pair_f64(ready_report, 0, "kinship_coefficient")?,
            EXPECTED_READY_CASE_KINSHIP_COEFFICIENT,
        )
        && required_pair_field(ready_report, 0, "relationship_label")?
            == EXPECTED_READY_CASE_RELATIONSHIP_LABEL)
}

pub(super) fn check_ready_case_summary_ready(ready_summary: &serde_json::Value) -> Result<bool> {
    Ok(required_string(ready_summary, "schema_version")? == EXPECTED_SUMMARY_SCHEMA_VERSION
        && required_string(ready_summary, "stage_id")? == EXPECTED_STAGE_ID
        && required_string(ready_summary, "method")? == EXPECTED_METHOD
        && required_string(ready_summary, "reference_panel")? == EXPECTED_REFERENCE_PANEL
        && required_string(ready_summary, "reference_build")? == EXPECTED_REFERENCE_BUILD
        && required_string(ready_summary, "population_scope")? == EXPECTED_POPULATION_SCOPE
        && required_string(ready_summary, "status")? == EXPECTED_READY_STATUS
        && required_u64(ready_summary, "pair_count")? == EXPECTED_READY_CASE_PAIR_COUNT
        && required_u64(ready_summary, "observed_max_overlap_snps")?
            == EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS)
}

pub(super) fn check_ready_case_pairwise_table_ready(ready_segments: &str) -> bool {
    ready_segments.contains(EXPECTED_PAIRWISE_TABLE_ROW)
}

pub(super) fn check_ready_case_stage_metrics_ready(
    ready_stage_metrics: &serde_json::Value,
) -> Result<bool> {
    Ok(required_string(ready_stage_metrics, "schema_version")?
        == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && required_string(ready_stage_metrics, "status")? == EXPECTED_READY_STATUS
        && required_u64(ready_stage_metrics, "pair_count")? == EXPECTED_READY_CASE_PAIR_COUNT
        && required_bool(ready_stage_metrics, "expectation_matched")?)
}

pub(super) fn check_insufficient_case_report_ready(
    insufficient_report: &serde_json::Value,
) -> Result<bool> {
    Ok(required_string(insufficient_report, "schema_version")? == EXPECTED_REPORT_SCHEMA_VERSION
        && required_string(insufficient_report, "status")? == EXPECTED_INSUFFICIENT_STATUS
        && required_u64(insufficient_report, "pair_count")?
            == EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT
        && required_u64(insufficient_report, "observed_max_overlap_snps")?
            == EXPECTED_INSUFFICIENT_CASE_OBSERVED_MAX_OVERLAP_SNPS
        && optional_string(insufficient_report, "insufficiency_reason")?.as_deref()
            == Some(EXPECTED_INSUFFICIENCY_REASON)
        && required_pair_array_len(insufficient_report)? == 0)
}

pub(super) fn check_insufficient_case_summary_ready(
    insufficient_summary: &serde_json::Value,
) -> Result<bool> {
    Ok(required_string(insufficient_summary, "schema_version")? == EXPECTED_SUMMARY_SCHEMA_VERSION
        && required_string(insufficient_summary, "stage_id")? == EXPECTED_STAGE_ID
        && required_string(insufficient_summary, "method")? == EXPECTED_METHOD
        && required_string(insufficient_summary, "reference_panel")? == EXPECTED_REFERENCE_PANEL
        && required_string(insufficient_summary, "reference_build")? == EXPECTED_REFERENCE_BUILD
        && required_string(insufficient_summary, "population_scope")? == EXPECTED_POPULATION_SCOPE
        && required_string(insufficient_summary, "status")? == EXPECTED_INSUFFICIENT_STATUS
        && required_u64(insufficient_summary, "pair_count")?
            == EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT
        && required_u64(insufficient_summary, "observed_max_overlap_snps")?
            == EXPECTED_INSUFFICIENT_CASE_OBSERVED_MAX_OVERLAP_SNPS
        && optional_string(insufficient_summary, "insufficiency_reason")?.as_deref()
            == Some(EXPECTED_INSUFFICIENCY_REASON))
}

pub(super) fn check_insufficient_case_stage_metrics_ready(
    insufficient_stage_metrics: &serde_json::Value,
) -> Result<bool> {
    Ok(required_string(insufficient_stage_metrics, "schema_version")?
        == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && required_string(insufficient_stage_metrics, "status")? == EXPECTED_INSUFFICIENT_STATUS
        && required_u64(insufficient_stage_metrics, "pair_count")?
            == EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT
        && required_bool(insufficient_stage_metrics, "expectation_matched")?
        && optional_string(insufficient_stage_metrics, "insufficiency_reason")?.as_deref()
            == Some(EXPECTED_INSUFFICIENCY_REASON))
}

pub(super) fn check_parser_smoke_ready(
    parser_row: &ParserObservation,
    parser_smoke_method: &str,
    parser_smoke_status: &str,
    parser_smoke_pair_count: u64,
    parser_smoke_observed_max_overlap_snps: u64,
) -> bool {
    parser_row.passed
        && parser_row.parsed_schema_version == EXPECTED_PARSER_SCHEMA_VERSION
        && parser_smoke_method == EXPECTED_METHOD
        && parser_smoke_status == EXPECTED_READY_STATUS
        && parser_smoke_pair_count == EXPECTED_READY_CASE_PAIR_COUNT
        && parser_smoke_observed_max_overlap_snps == EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS
}

pub(super) fn check_bam_pipeline_ready(
    bam_pipeline: &PipelineObservation,
    bam_pipeline_node: &PipelineNode,
) -> bool {
    bam_pipeline.valid
        && bam_pipeline.pipeline_id == EXPECTED_BAM_PIPELINE_ID
        && sorted_strings(&bam_pipeline_node.depends_on)
            == sorted_strings_from_slice(&REQUIRED_BAM_PIPELINE_DEPENDENCIES)
        && sorted_strings(&bam_pipeline_node.external_inputs)
            == sorted_strings_from_slice(&REQUIRED_BAM_PIPELINE_EXTERNAL_INPUTS)
        && sorted_strings(&bam_pipeline_node.upstream_inputs)
            == sorted_strings_from_slice(&REQUIRED_BAM_PIPELINE_UPSTREAM_INPUTS)
        && sorted_strings(&bam_pipeline_node.outputs)
            == sorted_strings_from_slice(&REQUIRED_BAM_PIPELINE_OUTPUTS)
}

pub(super) fn check_bam_locality_ready(bam_pipeline: &PipelineObservation) -> bool {
    bam_pipeline.nodes.iter().all(|node| {
        node.stage_id == EXPECTED_STAGE_ID
            || !node.upstream_inputs.iter().any(|input| input == "overlap_correction_summary_json")
    })
}

pub(super) fn check_vcf_isolation_ready(vcf_pipeline: &PipelineObservation) -> bool {
    vcf_pipeline.valid
        && vcf_pipeline.pipeline_id == EXPECTED_VCF_PIPELINE_ID
        && !vcf_pipeline.nodes.iter().any(|node| {
            node.stage_id == EXPECTED_STAGE_ID
                || node.upstream_inputs.iter().any(|input| {
                    input == "overlap_correction_summary_json" || input == "kinship_report_json"
                })
        })
}
