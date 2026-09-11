#![cfg(test)]
use super::{checks::*, declarations::*, input::*};
use crate::downstream::observations::{ParserObservation, PipelineNode, PipelineObservation};
use anyhow::Result;
use serde_json::json;
#[test]
fn insufficient_pairs_require_explicit_reason_and_empty_results() -> Result<()> {
    let mut report = json!({"schema_version":EXPECTED_REPORT_SCHEMA_VERSION,"status":EXPECTED_INSUFFICIENT_STATUS,"pair_count":0,"observed_max_overlap_snps":EXPECTED_INSUFFICIENT_CASE_OBSERVED_MAX_OVERLAP_SNPS,"insufficiency_reason":EXPECTED_INSUFFICIENCY_REASON,"pairwise_results":[]});
    assert!(check_insufficient_case_report_ready(&report)?);
    report["pairwise_results"] = json!([{"sample_a":"a","sample_b":"b"}]);
    assert!(!check_insufficient_case_report_ready(&report)?);
    report["pairwise_results"] = json!([]);
    report["insufficiency_reason"] = json!(null);
    assert!(!check_insufficient_case_report_ready(&report)?);
    Ok(())
}
#[test]
fn required_snapshot_counts_reject_null_strings_and_negative_values() {
    let mut row = ParserObservation {
        representative_stage_id: EXPECTED_STAGE_ID.into(),
        representative_tool_id: EXPECTED_METHOD.into(),
        proof_path: "parser.json".into(),
        parsed_schema_version: EXPECTED_PARSER_SCHEMA_VERSION.into(),
        normalized_snapshot: std::collections::BTreeMap::new(),
        passed: true,
    };
    for malformed in [json!(null), json!("1"), json!(-1), json!(1.5)] {
        row.normalized_snapshot.insert("pair_count".into(), malformed);
        assert!(required_snapshot_u64(&row, "pair_count").is_err());
    }
}
#[test]
fn vcf_pipeline_refuses_bam_kinship_evidence_leaks() {
    let node = PipelineNode {
        stage_id: "vcf.filter".into(),
        depends_on: vec![],
        external_inputs: vec![],
        upstream_inputs: vec![],
        outputs: vec![],
    };
    let mut report = PipelineObservation {
        output_path: "pipeline.json".into(),
        pipeline_id: EXPECTED_VCF_PIPELINE_ID.into(),
        valid: true,
        nodes: vec![node],
    };
    assert!(check_vcf_isolation_ready(&report));
    report.nodes[0].upstream_inputs.push("kinship_report_json".into());
    assert!(!check_vcf_isolation_ready(&report));
    report.nodes[0].upstream_inputs.clear();
    report.nodes[0].stage_id = EXPECTED_STAGE_ID.into();
    assert!(!check_vcf_isolation_ready(&report));
}
#[test]
fn overlap_summary_is_local_to_kinship_node() {
    let node = PipelineNode {
        stage_id: EXPECTED_STAGE_ID.into(),
        depends_on: vec![],
        external_inputs: vec![],
        upstream_inputs: vec!["overlap_correction_summary_json".into()],
        outputs: vec![],
    };
    let mut report = PipelineObservation {
        output_path: "pipeline.json".into(),
        pipeline_id: EXPECTED_BAM_PIPELINE_ID.into(),
        valid: true,
        nodes: vec![node],
    };
    assert!(check_bam_locality_ready(&report));
    report.nodes[0].stage_id = "bam.sex".into();
    assert!(!check_bam_locality_ready(&report));
}
