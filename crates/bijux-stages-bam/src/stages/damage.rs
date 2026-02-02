use std::path::Path;

use bijux_core::{CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
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
    let outputs =
        crate::stages::support::audit_outputs(bijux_domain_bam::BamStage::Damage, out_dir);
    let out_json = out_dir.join("damage.pydamage.json");
    let command = match tool.tool_id.0.as_str() {
        "mapdamage2" => crate::tools::mapdamage2::damage_args(bam, out_dir, params),
        _ => crate::tools::pydamage::args(bam, &out_json, params),
    };
    let plan = StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: command,
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![bijux_core::ArtifactRef {
                name: "bam".to_string(),
                path: bam.to_path_buf(),
            }],
            outputs,
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
        effective_params: crate::stages::support::ensure_effective_params(
            serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
        )?,
        aux_images: std::collections::BTreeMap::new(),
    };
    crate::stages::support::ensure_required_outputs(
        plan,
        &[
            "damage_report",
            "damage_pydamage",
            "damage_mapdamage2",
            "damage_profiler",
            "summary",
            "stage_metrics",
        ],
    )
}
