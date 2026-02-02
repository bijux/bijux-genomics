use std::path::Path;

use bijux_core::{CommandSpecV1, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_bam::params::FilterEffectiveParams;

pub const STAGE_ID: &str = bijux_domain_bam::BamStage::Filter.as_str();
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// # Errors
/// Returns an error if required outputs are missing from the plan.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    bam: &Path,
    out_dir: &Path,
    params: &FilterEffectiveParams,
) -> anyhow::Result<StagePlanV1> {
    let outputs =
        crate::stages::support::audit_outputs(bijux_domain_bam::BamStage::Filter, out_dir);
    let out_bam = out_dir.join("filtered.bam");
    let plan = StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: crate::tools::samtools::filter_args(bam, params, &out_bam),
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
            "mapq_threshold": params.mapq_threshold,
            "include_flags": params.include_flags,
            "exclude_flags": params.exclude_flags,
            "min_length": params.min_length,
            "remove_duplicates": params.remove_duplicates,
            "base_quality_threshold": params.base_quality_threshold,
        }),
        effective_params: crate::stages::support::ensure_effective_params(
            serde_json::to_value(params).unwrap_or(serde_json::Value::Null),
        )?,
        aux_images: std::collections::BTreeMap::new(),
    };
    crate::stages::support::ensure_required_outputs(
        plan,
        &[
            "filtered_bam",
            "filtered_bai",
            "flagstat",
            "idxstats",
            "summary",
            "stage_metrics",
        ],
    )
}
