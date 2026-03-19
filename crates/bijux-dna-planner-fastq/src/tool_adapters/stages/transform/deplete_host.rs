use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{screen::ScreenEffectiveParams, PairedMode};
use bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_HOST;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_DEPLETE_HOST;
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
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_host_depletion_tool_list(std::slice::from_ref(&tool_id))?;
    let report = out_dir.join("host_depletion_report.json");
    let paired_mode = if r2.is_some() {
        PairedMode::PairedEnd
    } else {
        PairedMode::SingleEnd
    };
    let effective_params = ScreenEffectiveParams {
        paired_mode,
        threads: tool.resources.threads,
        contaminant_db: Some("host_reference".to_string()),
    };
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    let mut outputs = Vec::new();
    let mut params = serde_json::json!({
        "tool": tool.tool_id.0,
        "input_r1": r1,
        "report_json": report,
    });
    if let Some(r2) = r2 {
        let output_r1 = out_dir.join("host_depleted_R1.fastq.gz");
        let output_r2 = out_dir.join("host_depleted_R2.fastq.gz");
        inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("host_depleted_reads_r1"),
            output_r1.clone(),
            ArtifactRole::Reads,
        ));
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("host_depleted_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
        params["input_r2"] = serde_json::json!(r2);
        params["output_r1"] = serde_json::json!(output_r1);
        params["output_r2"] = serde_json::json!(output_r2);
    } else {
        let output = out_dir.join("host_depleted.fastq.gz");
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("host_depleted_reads_r1"),
            output.clone(),
            ArtifactRole::Reads,
        ));
        params["output"] = serde_json::json!(output);
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("host_depletion_report_json"),
        report.clone(),
        ArtifactRole::ReportJson,
    ));
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
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params,
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize host depletion effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}
