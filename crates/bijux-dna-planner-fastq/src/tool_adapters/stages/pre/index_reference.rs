use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::reference_index::{
    ReferenceIndexEffectiveParams, INDEX_REFERENCE_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::stages::ids::STAGE_INDEX_REFERENCE;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_INDEX_REFERENCE;
pub const STAGE_VERSION: StageVersion = StageVersion(1);
pub type IndexReferencePlanOptions = crate::IndexReferenceStageParams;

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
    plan_with_options(tool, reference_fasta, out_dir, &IndexReferencePlanOptions::default())
}

pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    reference_fasta: &Path,
    out_dir: &Path,
    options: &IndexReferencePlanOptions,
) -> Result<StagePlanV1> {
    let output = reference_index_output(&tool.tool_id.0, out_dir)?;
    let report_json = out_dir.join("index_reference_report.json");
    let threads = options.threads.unwrap_or(tool.resources.threads);
    let effective_params = ReferenceIndexEffectiveParams {
        schema_version: INDEX_REFERENCE_SCHEMA_VERSION.to_string(),
        threads,
        index_format: tool.tool_id.to_string(),
        output_artifact: "reference_index".to_string(),
        report_artifact: "report_json".to_string(),
        index_prefix: reference_index_prefix(&tool.tool_id.0).map(str::to_string),
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
        command: CommandSpecV1 {
            template: index_reference_command(
                &tool.tool_id.0,
                reference_fasta,
                &output,
                threads,
            )?,
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
            ), ArtifactRef::required(
                ArtifactId::from_static("report_json"),
                report_json.clone(),
                ArtifactRole::ReportJson,
            )],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "reference_fasta": reference_fasta,
            "out_dir": out_dir,
            "reference_index": output,
            "report_json": report_json,
            "threads": threads,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize index reference effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn reference_index_prefix(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "bowtie2_build" => Some("reference"),
        "star" => None,
        _ => None,
    }
}

fn reference_index_output(tool_id: &str, out_dir: &Path) -> Result<std::path::PathBuf> {
    match tool_id {
        "bowtie2_build" => Ok(out_dir
            .join("reference_index")
            .join("bowtie2")
            .join("reference")),
        "star" => Ok(out_dir.join("reference_index").join("star")),
        _ => Err(anyhow!(
            "unsupported reference indexing tool for stage planning: {tool_id}"
        )),
    }
}

fn index_reference_command(
    tool_id: &str,
    reference_fasta: &Path,
    output: &Path,
    threads: u32,
) -> Result<Vec<String>> {
    match tool_id {
        "bowtie2_build" => Ok(vec![
            "bowtie2-build".to_string(),
            reference_fasta.display().to_string(),
            output.display().to_string(),
        ]),
        "star" => Ok(vec![
            "STAR".to_string(),
            "--runMode".to_string(),
            "genomeGenerate".to_string(),
            "--runThreadN".to_string(),
            threads.to_string(),
            "--genomeDir".to_string(),
            output.display().to_string(),
            "--genomeFastaFiles".to_string(),
            reference_fasta.display().to_string(),
        ]),
        _ => Err(anyhow!(
            "unsupported reference indexing tool for stage planning: {tool_id}"
        )),
    }
}
