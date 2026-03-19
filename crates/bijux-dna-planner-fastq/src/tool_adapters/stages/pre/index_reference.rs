use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_INDEX_REFERENCE;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_INDEX_REFERENCE;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_index_reference_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
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

pub fn plan(
    tool: &ToolExecutionSpecV1,
    reference_fasta: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let output = out_dir.join("reference_index");
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 {
            template: tool.command.template.to_vec(),
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reference_fasta"),
                reference_fasta.to_path_buf(),
                ArtifactRole::Reference,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reference_index"),
                output.clone(),
                ArtifactRole::Index,
            )],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "reference_fasta": reference_fasta,
            "out_dir": out_dir,
            "reference_index": output,
        }),
        effective_params: serde_json::json!({
            "threads": tool.resources.threads,
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}
