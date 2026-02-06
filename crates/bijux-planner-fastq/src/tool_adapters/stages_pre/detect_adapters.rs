use std::path::Path;

use bijux_core::{StageId, StageVersion, ToolExecutionSpecV1};
use bijux_domain_fastq::params::{detect_adapters::DetectAdaptersEffectiveParams, PairedMode};
use bijux_domain_fastq::STAGE_DETECT_ADAPTERS;
use bijux_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_DETECT_ADAPTERS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn plan(tool: &ToolExecutionSpecV1, r1: &Path, out_dir: &Path) -> StagePlanV1 {
    let effective_params = DetectAdaptersEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: tool.resources.threads,
        sample_reads: Some(100_000),
    };
    StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                "reads_r1",
                r1.to_path_buf(),
                bijux_core::ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                "fastqc_dir",
                out_dir.join("fastqc"),
                bijux_core::ArtifactRole::Index,
            )],
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
        reason: bijux_stage_contract::PlanDecisionReason::default(),
    }
}
