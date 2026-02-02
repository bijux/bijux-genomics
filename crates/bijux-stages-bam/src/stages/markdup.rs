use std::path::Path;

use bijux_core::{CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_bam::params::MarkDupEffectiveParams;

pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Markdup.as_str();
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// # Errors
/// Returns an error if required outputs are missing from the plan.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    bam: &Path,
    out_dir: &Path,
    params: &MarkDupEffectiveParams,
) -> anyhow::Result<StagePlanV1> {
    let outputs =
        crate::stages::support::audit_outputs(bijux_domain_bam::BamStage::Markdup, out_dir);
    let out_bam = out_dir.join("markdup.bam");
    let flagstat = out_dir.join("flagstat.txt");
    let idxstats = out_dir.join("idxstats.txt");
    let command = match tool.tool_id.0.as_str() {
        "samtools" => crate::tools::samtools::markdup_args(
            bam, &out_bam, &flagstat, &idxstats, params,
        ),
        _ => crate::tools::gatk::markdup_args(bam, &out_bam, &flagstat, &idxstats, params),
    };
    let plan = StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 { template: command },
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
            "optical_duplicates": params.optical_duplicates,
            "umi_policy": params.umi_policy,
            "duplicate_action": params.duplicate_action,
        }),
        effective_params: crate::stages::support::ensure_effective_params(
            serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
        )?,
        aux_images: std::collections::BTreeMap::new(),
    };
    crate::stages::support::ensure_required_outputs(
        plan,
        &[
            "markdup_bam",
            "markdup_bai",
            "flagstat",
            "idxstats",
            "summary",
            "stage_metrics",
        ],
    )
}
