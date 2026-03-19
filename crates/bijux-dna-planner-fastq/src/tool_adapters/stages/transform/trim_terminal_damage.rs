use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_TERMINAL_DAMAGE;
use bijux_dna_stage_contract::{ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_TRIM_TERMINAL_DAMAGE;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

fn output_name(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "cutadapt" => Some("trim_terminal_damage.cutadapt.fastq.gz"),
        "seqkit" => Some("trim_terminal_damage.seqkit.fastq.gz"),
        _ => None,
    }
}

/// # Errors
/// Returns an error when the tool does not support `fastq.trim_terminal_damage`.
pub fn plan_trim_terminal_damage(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let out_name = output_name(tool.tool_id.as_str())
        .ok_or_else(|| anyhow!("unsupported trim_terminal_damage tool {}", tool.tool_id))?;
    let output = out_dir.join(out_name);
    let report = out_dir.join("trim_terminal_damage_report.json");
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: tool.command.template.to_vec(),
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reads_r1"),
                r1.to_path_buf(),
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("trimmed_reads"),
                    output.clone(),
                    ArtifactRole::TrimmedReads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("report_json"),
                    report.clone(),
                    ArtifactRole::MetricsJson,
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
        effective_params: serde_json::json!({
            "damage_mode": "ancient",
            "trim_5p_bases": 2,
            "trim_3p_bases": 2,
            "transition_masking": "CT_GA_terminal_windows",
            "udg_classification_source": "config_or_inferred",
            "threads": tool.resources.threads,
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(
            PlanReasonKind::Default,
            "damage-aware terminal trimming",
        ),
    })
}
