use std::path::Path;

use bijux_core::{ArtifactRef, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_bam::params::DamageEffectiveParams;

pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Damage.as_str();
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// # Errors
/// Returns an error if required outputs are missing from the plan.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    bam: &Path,
    out_dir: &Path,
    params: &DamageEffectiveParams,
) -> anyhow::Result<StagePlanV1> {
    let plan = StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "bam".to_string(),
                path: bam.to_path_buf(),
            }],
            outputs: vec![
                ArtifactRef {
                    name: "damage_report".to_string(),
                    path: out_dir.join("damage.json"),
                },
                ArtifactRef {
                    name: "damage_pydamage".to_string(),
                    path: out_dir.join("damage.pydamage.json"),
                },
                ArtifactRef {
                    name: "damage_profiler".to_string(),
                    path: out_dir.join("damage.profiler.json"),
                },
                ArtifactRef {
                    name: "damage_metrics".to_string(),
                    path: out_dir.join("damage.metrics.json"),
                },
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "bam": bam,
            "udg_model": params.udg_model,
            "pmd_threshold_5p": params.pmd_threshold_5p,
            "pmd_threshold_3p": params.pmd_threshold_3p,
            "trim_5p": params.trim_5p,
            "trim_3p": params.trim_3p,
        }),
        effective_params: serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
        aux_images: std::collections::BTreeMap::new(),
    };
    super::ensure_required_outputs(
        plan,
        &[
            "damage_report",
            "damage_pydamage",
            "damage_profiler",
            "damage_metrics",
        ],
    )
}
