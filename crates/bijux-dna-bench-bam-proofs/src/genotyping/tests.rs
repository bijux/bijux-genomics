#![cfg(test)]
use super::{checks::*, declarations::*};
use crate::downstream::observations::{
    ParserObservation, PipelineNode, PipelineObservation, VariantCounts, VariantMissingness,
};
#[test]
fn missingness_requires_observations_and_finite_thresholds() {
    let mut row = VariantMissingness {
        stage_id: "vcf.qc".into(),
        tool_id: EXPECTED_VCF_QC_TOOL_ID.into(),
        coverage_status: "complete".into(),
        sample_missingness_count: 2,
        variant_missingness_count: 4,
        sample_exclusion_threshold: EXPECTED_SAMPLE_MISSINGNESS_THRESHOLD,
        variant_exclusion_threshold: EXPECTED_VARIANT_MISSINGNESS_THRESHOLD,
    };
    assert!(check_downstream_missingness_ready(&row));
    row.sample_missingness_count = 0;
    assert!(!check_downstream_missingness_ready(&row));
    row.sample_missingness_count = 2;
    row.variant_exclusion_threshold = f64::NAN;
    assert!(!check_downstream_missingness_ready(&row));
}
#[test]
fn downstream_counts_reject_wrong_sample_cardinality() {
    let mut row = VariantCounts {
        stage_id: "vcf.stats".into(),
        tool_id: EXPECTED_VCF_STATS_TOOL_ID.into(),
        coverage_status: "complete".into(),
        variant_count: EXPECTED_VARIANT_COUNT,
        sample_count: EXPECTED_SAMPLE_COUNT,
    };
    assert!(check_downstream_variant_metrics_ready(&row));
    row.sample_count += 1;
    assert!(!check_downstream_variant_metrics_ready(&row));
}
#[test]
fn producer_handoff_preserves_output_order_and_multiplicity() {
    let mut node = PipelineNode {
        stage_id: EXPECTED_STAGE_ID.into(),
        depends_on: vec![],
        external_inputs: vec![],
        upstream_inputs: vec![],
        outputs: REQUIRED_BAM_PIPELINE_OUTPUTS.iter().map(|s| (*s).into()).collect(),
    };
    let report = PipelineObservation {
        output_path: "pipeline.json".into(),
        pipeline_id: EXPECTED_BAM_PIPELINE_ID.into(),
        valid: true,
        nodes: vec![],
    };
    assert!(check_bam_pipeline_ready(&report, &node));
    node.outputs.swap(0, 1);
    assert!(!check_bam_pipeline_ready(&report, &node));
    node.outputs.swap(0, 1);
    node.outputs.push(node.outputs[0].clone());
    assert!(!check_bam_pipeline_ready(&report, &node));
}
#[test]
fn parser_metrics_reject_nonfinite_values_and_reference_drift() {
    let row = ParserObservation {
        representative_stage_id: EXPECTED_STAGE_ID.into(),
        representative_tool_id: EXPECTED_TOOL_ID.into(),
        proof_path: "parser.json".into(),
        parsed_schema_version: EXPECTED_PARSER_SCHEMA_VERSION.into(),
        normalized_snapshot: std::collections::BTreeMap::new(),
        passed: true,
    };
    assert!(check_parser_smoke_ready(
        &row,
        EXPECTED_PARSER_CALL_RATE,
        EXPECTED_PARSER_MEAN_POSTERIOR,
        EXPECTED_REFERENCE_FASTA,
        EXPECTED_SITES_VCF
    ));
    assert!(!check_parser_smoke_ready(
        &row,
        f64::NAN,
        EXPECTED_PARSER_MEAN_POSTERIOR,
        EXPECTED_REFERENCE_FASTA,
        EXPECTED_SITES_VCF
    ));
    assert!(!check_parser_smoke_ready(
        &row,
        EXPECTED_PARSER_CALL_RATE,
        EXPECTED_PARSER_MEAN_POSTERIOR,
        "foreign-reference",
        EXPECTED_SITES_VCF
    ));
}
