use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{umi::FastqUmiParams, umi::UMI_SCHEMA_VERSION, PairedMode};
use bijux_dna_domain_fastq::STAGE_EXTRACT_UMIS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_EXTRACT_UMIS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);
const DEFAULT_UMI_PATTERN: &str = "NNNNNNNN";

pub fn normalize_umi_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

/// Build a UMI plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_umi(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    umi_pattern: Option<&str>,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_umi_tool_list(std::slice::from_ref(&tool_id))?;
    let output_r1 = out_dir.join("umi_tools.r1.fastq.gz");
    let output_r2 = out_dir.join("umi_tools.r2.fastq.gz");
    let report_json = out_dir.join("umi_report.json");
    let umi_pattern = umi_pattern.unwrap_or(DEFAULT_UMI_PATTERN);
    let effective_params = FastqUmiParams {
        schema_version: UMI_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: tool.resources.threads,
        umi_pattern: Some(umi_pattern.to_string()),
    };
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: Some(crate::tool_adapters::default_stage_instance_id(
            &STAGE_ID,
            &tool.tool_id,
        )),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 {
            template: tool.command.template.to_vec(),
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("reads_r1"),
                    r1.to_path_buf(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("reads_r2"),
                    r2.to_path_buf(),
                    ArtifactRole::Reads,
                ),
            ],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("umi_reads_r1"),
                    output_r1.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("umi_reads_r2"),
                    output_r2.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::optional(
                    ArtifactId::from_static("report_json"),
                    report_json.clone(),
                    ArtifactRole::MetricsJson,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "r1": r1,
            "r2": r2,
            "out_dir": out_dir,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report_json,
            "umi_pattern": umi_pattern
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize umi effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn normalize_tools_with_allowlist(
    tools: &[String],
    allowlist: &[bijux_dna_core::ids::ToolId],
) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}
