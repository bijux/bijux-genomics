use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    correct::{CorrectionEngine, FastqCorrectParams, QualityEncoding, CORRECT_SCHEMA_VERSION},
    PairedMode,
};
use bijux_dna_domain_fastq::STAGE_CORRECT_ERRORS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_CORRECT_ERRORS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_correct_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

/// Build a correct plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_correct(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_correct_tool_list(std::slice::from_ref(&tool_id))?;
    let output_r1 = out_dir.join("reads_r1.fastq.gz");
    let output_r2 = r2.map(|_| out_dir.join("reads_r2.fastq.gz"));
    let report_json = out_dir.join("correct_report.json");
    let effective_params = FastqCorrectParams {
        schema_version: CORRECT_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::from_has_r2(r2.is_some()),
        threads: tool.resources.threads,
        correction_engine: correction_engine_for_tool(&tool.tool_id.0)?,
        quality_encoding: QualityEncoding::Phred33,
        kmer_size: None,
        max_memory_gb: None,
        trusted_kmer_artifact: None,
        conservative_mode: false,
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
            template: crate::tool_adapters::template_render::render_command_template(
                &tool.command.template,
                &[
                    ("reads", Some(r1.display().to_string())),
                    ("reads_r1", Some(r1.display().to_string())),
                    ("reads_r2", r2.map(|path| path.display().to_string())),
                    ("corrected_reads_r1", Some(output_r1.display().to_string())),
                    (
                        "corrected_reads_r2",
                        output_r2.as_ref().map(|path| path.display().to_string()),
                    ),
                    ("report_json", Some(report_json.display().to_string())),
                    ("threads", Some(tool.resources.threads.to_string())),
                ],
            )?,
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: {
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
                inputs
            },
            outputs: {
                let mut outputs = vec![ArtifactRef::required(
                    ArtifactId::from_static("corrected_reads_r1"),
                    output_r1.clone(),
                    ArtifactRole::Reads,
                )];
                if let Some(output_r2) = &output_r2 {
                    outputs.push(ArtifactRef::required(
                        ArtifactId::from_static("corrected_reads_r2"),
                        output_r2.clone(),
                        ArtifactRole::Reads,
                    ));
                }
                outputs.push(ArtifactRef::required(
                    ArtifactId::from_static("report_json"),
                    report_json.clone(),
                    ArtifactRole::ReportJson,
                ));
                outputs
            },
        },
        out_dir: out_dir.to_path_buf(),
        params: {
            let mut params = serde_json::json!({
                "tool": tool.tool_id.0,
                "r1": r1,
                "out_dir": out_dir,
                "output_r1": output_r1,
                "report_json": report_json,
            });
            if let Some((r2, output_r2)) = r2.zip(output_r2.as_ref()) {
                params["r2"] = serde_json::json!(r2);
                params["output_r2"] = serde_json::json!(output_r2);
            }
            params
        },
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize correct effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn correction_engine_for_tool(tool_id: &str) -> Result<CorrectionEngine> {
    match tool_id {
        "rcorrector" => Ok(CorrectionEngine::Rcorrector),
        "musket" => Ok(CorrectionEngine::Musket),
        "lighter" => Ok(CorrectionEngine::Lighter),
        "bayeshammer" => Ok(CorrectionEngine::Bayeshammer),
        _ => Err(anyhow!("unsupported tool: {tool_id}")),
    }
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
