use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_POLYG_TAILING;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_POLYG_TAILING;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

fn output_name(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "fastp" => Some("polyg.fastp.fastq.gz"),
        "bbduk" => Some("polyg.bbduk.fastq.gz"),
        _ => None,
    }
}

/// Build a polyG tail trimming plan.
///
/// # Errors
/// Returns an error if the tool is unsupported for this stage.
pub fn plan_polyg_tailing(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let out_name = output_name(tool.tool_id.as_str())
        .ok_or_else(|| anyhow!("unsupported polyg_tailing tool {}", tool.tool_id))?;
    let output = out_dir.join(out_name);
    let report = out_dir.join("polyg_tailing_report.json");
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
                    ArtifactId::from_static("reads_r1_polyg_trimmed"),
                    output.clone(),
                    ArtifactRole::TrimmedReads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("polyg_tailing_report_json"),
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
            "applicability": {
                "requires_illumina_like_cycle_artifacts": true,
                "skip_when_not_applicable": true
            },
            "threads": tool.resources.threads,
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::new(
            bijux_dna_stage_contract::PlanReasonKind::Default,
            "polyG tail trimming",
        ),
    })
}
