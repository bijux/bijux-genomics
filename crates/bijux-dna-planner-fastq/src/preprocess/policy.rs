use bijux_dna_core::ids::ToolId;
use bijux_dna_core::prelude::StageId;
use bijux_dna_domain_fastq::{STAGE_DETECT_ADAPTERS, STAGE_TRIM_READS};

#[derive(Debug, Clone)]
pub struct PreprocessPolicyDecision {
    pub adapter_inference: Option<serde_json::Value>,
    pub adapter_bank_preset_override: Option<String>,
    pub pipeline_stages: Vec<StageId>,
    pub pipeline_tools: Vec<ToolId>,
    pub stage_skips: Vec<serde_json::Value>,
}

#[must_use]
pub fn apply_preprocess_policy(
    pipeline_stages: Vec<StageId>,
    pipeline_tools: Vec<ToolId>,
) -> PreprocessPolicyDecision {
    let adapter_inference = pipeline_stages
        .iter()
        .zip(pipeline_tools.iter())
        .find(|(stage, _)| stage == &&STAGE_DETECT_ADAPTERS)
        .map(|(stage, tool)| {
            let trim_binding = pipeline_stages
                .iter()
                .zip(pipeline_tools.iter())
                .find(|(candidate_stage, _)| candidate_stage == &&STAGE_TRIM_READS)
                .map(|(candidate_stage, candidate_tool)| {
                    serde_json::json!({
                        "stage_id": candidate_stage.as_str(),
                        "tool_id": candidate_tool.as_str(),
                    })
                });
            serde_json::json!({
                "schema_version": "bijux.fastq.preprocess_policy.v1",
                "source_stage_id": stage.as_str(),
                "source_tool_id": tool.as_str(),
                "evidence_artifacts": ["adapter_report", "adapter_evidence_dir"],
                "handoff_mode": "runtime_evidence",
                "consumer_binding": trim_binding,
            })
        });
    PreprocessPolicyDecision {
        adapter_inference,
        adapter_bank_preset_override: None,
        pipeline_stages,
        pipeline_tools,
        stage_skips: Vec::new(),
    }
}
