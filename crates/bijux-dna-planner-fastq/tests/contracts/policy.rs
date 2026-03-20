use bijux_dna_core::ids::{StageId, ToolId};

#[test]
fn preprocess_policy_keeps_filter_stage_when_trim_and_filter_share_fastp() {
    let decision = bijux_dna_planner_fastq::apply_preprocess_policy(
        vec![
            StageId::from_static("fastq.trim_reads"),
            StageId::from_static("fastq.filter_reads"),
        ],
        vec![
            ToolId::from_static("fastp"),
            ToolId::from_static("fastp"),
        ],
    );

    assert_eq!(
        decision
            .pipeline_stages
            .iter()
            .map(|stage| stage.as_str())
            .collect::<Vec<_>>(),
        vec!["fastq.trim_reads", "fastq.filter_reads"]
    );
    assert!(
        decision.stage_skips.is_empty(),
        "governed FASTQ preprocessing stages must stay explicit in the planned pipeline"
    );
}
