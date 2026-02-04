use std::path::Path;

use bijux_core::{ArtifactRef, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_fastq::params::{detect_adapters::DetectAdaptersEffectiveParams, PairedMode};

pub const STAGE_ID: &str = "fastq.detect_adapters";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn plan(tool: &ToolExecutionSpecV1, r1: &Path, out_dir: &Path) -> StagePlanV1 {
    let effective_params = DetectAdaptersEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: tool.resources.threads,
        sample_reads: Some(100_000),
    };
    StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: r1.to_path_buf(),
            }],
            outputs: vec![ArtifactRef {
                name: "fastqc_dir".to_string(),
                path: out_dir.join("fastqc"),
            }],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input": r1,
            "out_dir": out_dir,
            "sample_reads": effective_params.sample_reads,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .expect("serialize detect adapters effective params"),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_core::PlanDecisionReason::default(),
    }
}
