use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    screen::{
        ReferenceContaminantEffectiveParams, REFERENCE_DEPLETION_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_REFERENCE_CONTAMINANTS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_DEPLETE_REFERENCE_CONTAMINANTS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_contaminant_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

/// Build a contaminant screen plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_contaminant_screen(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_contaminant_screen_tool_list(std::slice::from_ref(&tool_id))?;
    let output_r1 = if r2.is_some() {
        out_dir.join("contaminant_screened_R1.fastq.gz")
    } else {
        out_dir.join("contaminant_screened.fastq.gz")
    };
    let output_r2 = r2.map(|_| out_dir.join("contaminant_screened_R2.fastq.gz"));
    let report = out_dir.join("contaminant_screen_report.json");
    let effective_params = ReferenceContaminantEffectiveParams {
        schema_version: REFERENCE_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: tool.resources.threads,
        contaminant_reference: "contaminant_reference".to_string(),
        index_artifact: "contaminant_reference_index".to_string(),
        retain_unmapped_pairs: r2.is_some(),
    };
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
        ArtifactId::from_static("contaminant_screened_reads"),
        output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("contaminant_screened_reads"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("contaminant_screen_report_json"),
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
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize contaminant screen effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}
