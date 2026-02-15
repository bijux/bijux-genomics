use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{screen::ScreenEffectiveParams, PairedMode};
use bijux_dna_domain_fastq::stages::ids::STAGE_HOST_DEPLETION;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_HOST_DEPLETION;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_host_depletion_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool {tool}"));
        }
    }
    Ok(normalized)
}

/// Build a host depletion plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_host_depletion(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_host_depletion_tool_list(std::slice::from_ref(&tool_id))?;
    let output = out_dir.join("host_depleted.fastq.gz");
    let report = out_dir.join("host_depletion_report.json");
    let effective_params = ScreenEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: tool.resources.threads,
        contaminant_db: Some("host_reference".to_string()),
    };
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reads_r1"),
                r1.to_path_buf(),
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("host_depleted_reads"),
                    output.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("host_depletion_report_json"),
                    report.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input": r1,
            "output": output,
            "report_json": report,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize host depletion effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}
