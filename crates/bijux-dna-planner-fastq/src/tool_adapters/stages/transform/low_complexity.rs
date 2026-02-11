use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::STAGE_LOW_COMPLEXITY;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_LOW_COMPLEXITY;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_low_complexity_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

fn low_complexity_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "dustmasker" => Some("dustmasker.fastq.gz"),
        "prinseq" => Some("prinseq_good.fastq"),
        "bbduk" => Some("bbduk.fastq.gz"),
        _ => None,
    }
}

/// Build a low-complexity filter plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_low_complexity(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let output_name = low_complexity_output_name(&tool.tool_id.0)
        .ok_or_else(|| anyhow!("unsupported low-complexity tool"))?;
    let output = out_dir.join(output_name);
    let report = out_dir.join("low_complexity_report.json");
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
                    ArtifactId::from_static("filtered_fastq"),
                    output.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("filter_report_json"),
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
        effective_params: serde_json::json!({}),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}
