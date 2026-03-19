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
    r2: Option<&Path>,
    out_dir: &Path,
    damage_mode: &str,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
) -> Result<StagePlanV1> {
    let out_name = output_name(tool.tool_id.as_str())
        .ok_or_else(|| anyhow!("unsupported trim_terminal_damage tool {}", tool.tool_id))?;
    let output_r1 = if r2.is_some() {
        out_dir.join(format!("R1.{out_name}"))
    } else {
        out_dir.join(out_name)
    };
    let output_r2 = r2.map(|_| out_dir.join(format!("R2.{out_name}")));
    let report = out_dir.join("trim_terminal_damage_report.json");
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
        ArtifactId::from_static("trimmed_reads"),
        output_r1.clone(),
        ArtifactRole::TrimmedReads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("trimmed_reads"),
            output_r2.clone(),
            ArtifactRole::TrimmedReads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        report.clone(),
        ArtifactRole::ReportJson,
    ));
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
        effective_params: serde_json::json!({
            "damage_mode": damage_mode,
            "trim_5p_bases": trim_5p_bases,
            "trim_3p_bases": trim_3p_bases,
            "paired_mode": if r2.is_some() { "paired_end" } else { "single_end" },
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
