use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    screen::{HostDepletionEffectiveParams, HOST_DEPLETION_SCHEMA_VERSION},
    PairedMode,
};
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
    reference_index: &Path,
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
    let effective_params = HostDepletionEffectiveParams {
        schema_version: HOST_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode,
        threads: tool.resources.threads,
        host_reference: "host_reference".to_string(),
        index_artifact: "host_reference_index".to_string(),
        retain_unmapped_pairs: r2.is_some(),
    };
    let mut inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    inputs.push(ArtifactRef::required(
        ArtifactId::from_static("reference_index"),
        reference_index.to_path_buf(),
        ArtifactRole::Index,
    ));
    let mut outputs = Vec::new();
    let mut params = serde_json::json!({
        "tool": tool.tool_id.0,
        "input_r1": r1,
        "reference_index": reference_index,
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
        command: CommandSpecV1 {
            template: host_depletion_command(
                &tool.tool_id.0,
                r1,
                r2,
                reference_index,
                out_dir,
                report.as_path(),
            )?,
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

fn host_depletion_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    reference_index: &Path,
    out_dir: &Path,
    report_json: &Path,
) -> Result<Vec<String>> {
    match tool_id {
        "bowtie2" => {
            let mut command = vec![
                "bowtie2".to_string(),
                "-x".to_string(),
                reference_index.display().to_string(),
                "--threads".to_string(),
                "1".to_string(),
                "-S".to_string(),
                "/dev/null".to_string(),
            ];
            if let Some(r2) = r2 {
                command.extend([
                    "-1".to_string(),
                    r1.display().to_string(),
                    "-2".to_string(),
                    r2.display().to_string(),
                    "--un-conc-gz".to_string(),
                    out_dir.join("host_depleted_R%.fastq.gz").display().to_string(),
                ]);
            } else {
                command.extend([
                    "-U".to_string(),
                    r1.display().to_string(),
                    "--un-gz".to_string(),
                    out_dir.join("host_depleted.fastq.gz").display().to_string(),
                ]);
            }
            command.extend([
                "--met-file".to_string(),
                report_json.display().to_string(),
            ]);
            Ok(command)
        }
        _ => Err(anyhow!(
            "unsupported host depletion tool for stage planning: {tool_id}"
        )),
    }
}
