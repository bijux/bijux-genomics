use super::contracts::approx_eq;
use super::contracts::declared_output_names;
use super::contracts::sorted_output_names;
use super::declarations::EXPECTED_BAM_PIPELINE_ID;
use super::declarations::EXPECTED_CROSS_PIPELINE_ID;
use super::declarations::EXPECTED_MIN_CALL_RATE;
use super::declarations::EXPECTED_MIN_POSTERIOR;
use super::declarations::EXPECTED_PARSER_CALL_RATE;
use super::declarations::EXPECTED_PARSER_MEAN_POSTERIOR;
use super::declarations::EXPECTED_PARSER_SCHEMA_VERSION;
use super::declarations::EXPECTED_REFERENCE_FASTA;
use super::declarations::EXPECTED_REGIONS;
use super::declarations::EXPECTED_SAMPLE_COUNT;
use super::declarations::EXPECTED_SAMPLE_ID;
use super::declarations::EXPECTED_SAMPLE_MISSINGNESS_THRESHOLD;
use super::declarations::EXPECTED_SITES_VCF;
use super::declarations::EXPECTED_TOOL_ID;
use super::declarations::EXPECTED_VARIANT_COUNT;
use super::declarations::EXPECTED_VARIANT_MISSINGNESS_THRESHOLD;
use super::declarations::EXPECTED_VCF_QC_TOOL_ID;
use super::declarations::EXPECTED_VCF_STATS_TOOL_ID;
use super::declarations::REQUIRED_BAM_PIPELINE_OUTPUTS;
use super::declarations::REQUIRED_PLAN_OUTPUT_IDS;
use super::declarations::REQUIRED_VCF_FILTER_UPSTREAM_INPUTS;
use super::observations::LocalGenotypingPlan;
use crate::downstream::observations::ParserObservation;
use crate::downstream::observations::PipelineNode;
use crate::downstream::observations::PipelineObservation;
use crate::downstream::observations::VariantCounts;
use crate::downstream::observations::VariantMissingness;
pub(super) fn check_local_plan_ready(
    local_plan: &LocalGenotypingPlan,
    genotyping_bcf_path: &str,
    genotyping_vcf_path: &str,
    command_shell: &str,
) -> bool {
    local_plan.params.sample_id == EXPECTED_SAMPLE_ID
        && local_plan.params.reference == EXPECTED_REFERENCE_FASTA
        && local_plan.params.sites == EXPECTED_SITES_VCF
        && local_plan.params.regions == EXPECTED_REGIONS
        && local_plan.params.caller == EXPECTED_TOOL_ID
        && local_plan.params.tool == EXPECTED_TOOL_ID
        && local_plan.effective_params.caller == EXPECTED_TOOL_ID
        && approx_eq(local_plan.effective_params.min_posterior, EXPECTED_MIN_POSTERIOR)
        && approx_eq(local_plan.effective_params.min_call_rate, EXPECTED_MIN_CALL_RATE)
        && command_shell.contains(EXPECTED_REFERENCE_FASTA)
        && command_shell.contains(EXPECTED_SITES_VCF)
        && command_shell.contains(EXPECTED_REGIONS)
        && command_shell.contains(genotyping_bcf_path)
        && command_shell.contains(genotyping_vcf_path)
}

pub(super) fn check_producer_contract_ready(
    local_plan: &LocalGenotypingPlan,
    genotyping_bcf_path: &str,
    genotyping_vcf_path: &str,
    genotyping_vcf_tbi_path: &str,
    genotyping_gl_path: &str,
) -> bool {
    local_plan.params.producer_contract.bcf == genotyping_bcf_path
        && local_plan.params.producer_contract.vcf == genotyping_vcf_path
        && local_plan.params.producer_contract.tbi == genotyping_vcf_tbi_path
        && local_plan.params.producer_contract.gl == genotyping_gl_path
        && declared_output_names(&local_plan.io.outputs)
            == sorted_output_names(&REQUIRED_PLAN_OUTPUT_IDS)
}

pub(super) fn check_parser_smoke_ready(
    parser_row: &ParserObservation,
    parser_smoke_call_rate: f64,
    parser_smoke_mean_posterior: f64,
    parser_smoke_reference_fasta: &str,
    parser_smoke_sites_vcf: &str,
) -> bool {
    parser_row.passed
        && parser_row.parsed_schema_version == EXPECTED_PARSER_SCHEMA_VERSION
        && approx_eq(parser_smoke_call_rate, EXPECTED_PARSER_CALL_RATE)
        && approx_eq(parser_smoke_mean_posterior, EXPECTED_PARSER_MEAN_POSTERIOR)
        && parser_smoke_reference_fasta == EXPECTED_REFERENCE_FASTA
        && parser_smoke_sites_vcf == EXPECTED_SITES_VCF
}

pub(super) fn check_bam_pipeline_ready(
    bam_pipeline: &PipelineObservation,
    bam_pipeline_node: &PipelineNode,
) -> bool {
    bam_pipeline.valid
        && bam_pipeline.pipeline_id == EXPECTED_BAM_PIPELINE_ID
        && bam_pipeline_node.outputs
            == REQUIRED_BAM_PIPELINE_OUTPUTS
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
}

pub(super) fn check_cross_pipeline_ready(
    cross_pipeline: &PipelineObservation,
    cross_pipeline_genotyping_node: &PipelineNode,
) -> bool {
    cross_pipeline.valid
        && cross_pipeline.pipeline_id == EXPECTED_CROSS_PIPELINE_ID
        && cross_pipeline_genotyping_node.outputs
            == REQUIRED_BAM_PIPELINE_OUTPUTS
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
}

pub(super) fn check_downstream_handoff_ready(
    cross_pipeline_vcf_filter_node: &PipelineNode,
) -> bool {
    cross_pipeline_vcf_filter_node.upstream_inputs
        == REQUIRED_VCF_FILTER_UPSTREAM_INPUTS
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>()
}

pub(super) fn check_downstream_variant_metrics_ready(stats_row: &VariantCounts) -> bool {
    stats_row.coverage_status == "complete"
        && stats_row.tool_id == EXPECTED_VCF_STATS_TOOL_ID
        && stats_row.variant_count == EXPECTED_VARIANT_COUNT
        && stats_row.sample_count == EXPECTED_SAMPLE_COUNT
}

pub(super) fn check_downstream_missingness_ready(qc_row: &VariantMissingness) -> bool {
    qc_row.coverage_status == "complete"
        && qc_row.tool_id == EXPECTED_VCF_QC_TOOL_ID
        && qc_row.sample_missingness_count != 0
        && qc_row.variant_missingness_count != 0
        && approx_eq(qc_row.sample_exclusion_threshold, EXPECTED_SAMPLE_MISSINGNESS_THRESHOLD)
        && approx_eq(qc_row.variant_exclusion_threshold, EXPECTED_VARIANT_MISSINGNESS_THRESHOLD)
}
