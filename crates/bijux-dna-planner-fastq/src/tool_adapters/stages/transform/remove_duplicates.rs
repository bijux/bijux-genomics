use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::STAGE_REMOVE_DUPLICATES;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_REMOVE_DUPLICATES;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_deduplicate_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

fn deduplicate_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fastuniq" => Some("fastuniq.fastq.gz"),
        "clumpify" => Some("clumpify.fastq.gz"),
        _ => None,
    }
}

/// Build a deduplicate plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_deduplicate(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let paired_mode = r2.is_some();
    let output_r1 = if paired_mode {
        out_dir.join(format!("{}.dedup.R1.fastq.gz", tool.tool_id))
    } else {
        let output_name = deduplicate_output_name(&tool.tool_id.0)
            .ok_or_else(|| anyhow!("unsupported deduplicate tool"))?;
        out_dir.join(output_name)
    };
    let output_r2 = r2.map(|_| out_dir.join(format!("{}.dedup.R2.fastq.gz", tool.tool_id)));
    let report = out_dir.join("deduplicate_report.json");
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    if let Some(r2) = r2 {
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("dedup_reads_r1"),
        output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("dedup_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        report.clone(),
        ArtifactRole::ReportJson,
    ));
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
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report,
        }),
        effective_params: serde_json::json!({}),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicate_output_name_rejects_unadmitted_tools() {
        assert!(deduplicate_output_name("prinseq").is_none());
    }
}
