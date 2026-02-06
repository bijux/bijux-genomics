use bijux_core::{StageId, StageVersion, ToolExecutionSpecV1};
use bijux_domain_fastq::params::{preprocess::PreprocessEffectiveParams, PairedMode};
use bijux_domain_fastq::STAGE_PREPROCESS;
use bijux_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_PREPROCESS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone)]
pub struct PreprocessPlan {
    pub r1: std::path::PathBuf,
    pub r2: Option<std::path::PathBuf>,
    pub stages: Vec<String>,
    pub enable_contaminant_removal: bool,
}

#[must_use]
pub fn plan_preprocess_stage(plan: &PreprocessPlan, tool: &ToolExecutionSpecV1) -> StagePlanV1 {
    let paired_mode = if plan.r2.is_some() {
        PairedMode::PairedEnd
    } else {
        PairedMode::SingleEnd
    };
    let effective_params = PreprocessEffectiveParams {
        enable_contaminant_removal: plan.enable_contaminant_removal,
        paired_mode,
        stages: plan.stages.clone(),
        threads: tool.resources.threads,
    };
    let out_dir = plan
        .r1
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("out");
    StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: {
                let mut inputs = Vec::new();
                inputs.push(ArtifactRef {
                    name: "reads_r1".to_string(),
                    path: plan.r1.clone(),
                });
                if let Some(r2) = plan.r2.as_ref() {
                    inputs.push(ArtifactRef {
                        name: "reads_r2".to_string(),
                        path: r2.clone(),
                    });
                }
                inputs
            },
            outputs: Vec::new(),
        },
        out_dir,
        params: serde_json::json!({
            "r1": plan.r1,
            "r2": plan.r2,
            "stages": plan.stages,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .expect("serialize preprocess effective params"),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_stage_contract::PlanDecisionReason::default(),
    }
}
