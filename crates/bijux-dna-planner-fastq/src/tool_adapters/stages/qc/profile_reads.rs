use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{validate::ValidateEffectiveParams, PairedMode};
use bijux_dna_domain_fastq::STAGE_PROFILE_READS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_PROFILE_READS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

/// Build a read profiling plan.
///
/// # Errors
/// Returns an error if planning fails.
pub fn plan_stats_neutral(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let effective_params = ValidateEffectiveParams {
        paired_mode: if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: tool.resources.threads,
        q_cutoff: None,
    };
    let command_template = profile_reads_command(&tool.tool_id.0, r1, r2)?;
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
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_instance_id: None,
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: CommandSpecV1 {
            template: command_template,
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs,
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("qc_json"),
                    out_dir.join("qc.json"),
                    ArtifactRole::MetricsJson,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("qc_tsv"),
                    out_dir.join("qc.tsv"),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::optional(
                    ArtifactId::from_static("qc_plots_dir"),
                    out_dir.join("plots"),
                    ArtifactRole::Index,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "out_dir": out_dir
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize profile_reads effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn profile_reads_command(tool_id: &str, r1: &Path, r2: Option<&Path>) -> Result<Vec<String>> {
    let mut command = match tool_id {
        "seqkit_stats" => vec![
            "seqkit".to_string(),
            "stats".to_string(),
            "-a".to_string(),
            "-T".to_string(),
            r1.display().to_string(),
        ],
        _ => {
            return Err(anyhow!(
                "unsupported read-profiling tool for stage planning: {tool_id}"
            ));
        }
    };
    if let Some(r2) = r2 {
        command.push(r2.display().to_string());
    }
    Ok(command)
}
