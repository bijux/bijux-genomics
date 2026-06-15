#![allow(clippy::expect_used)]

use bijux_dna_core::ids::{StageId, ToolId};

#[test]
fn preprocess_policy_keeps_filter_stage_when_trim_and_filter_share_fastp() {
    let decision = bijux_dna_planner_fastq::apply_preprocess_policy(
        vec![StageId::from_static("fastq.trim_reads"), StageId::from_static("fastq.filter_reads")],
        vec![ToolId::from_static("fastp"), ToolId::from_static("fastp")],
    );

    assert_eq!(
        decision.pipeline_stages.iter().map(StageId::as_str).collect::<Vec<_>>(),
        vec!["fastq.trim_reads", "fastq.filter_reads"]
    );
    assert!(
        decision.stage_skips.is_empty(),
        "governed FASTQ preprocessing stages must stay explicit in the planned pipeline"
    );
}

#[test]
fn preprocess_policy_records_adapter_handoff_when_detection_stage_is_present() {
    let decision = bijux_dna_planner_fastq::apply_preprocess_policy(
        vec![
            StageId::from_static("fastq.detect_adapters"),
            StageId::from_static("fastq.trim_reads"),
        ],
        vec![ToolId::from_static("fastqc"), ToolId::from_static("fastp")],
    );

    let adapter_inference = decision
        .adapter_inference
        .as_ref()
        .expect("detect_adapters should emit adapter handoff metadata");
    assert_eq!(
        adapter_inference["schema_version"],
        serde_json::json!("bijux.fastq.preprocess_policy.v1")
    );
    assert_eq!(adapter_inference["source_stage_id"], serde_json::json!("fastq.detect_adapters"));
    assert_eq!(adapter_inference["source_tool_id"], serde_json::json!("fastqc"));
    assert_eq!(
        adapter_inference["consumer_binding"]["stage_id"],
        serde_json::json!("fastq.trim_reads")
    );
    assert_eq!(adapter_inference["consumer_binding"]["tool_id"], serde_json::json!("fastp"));
}
