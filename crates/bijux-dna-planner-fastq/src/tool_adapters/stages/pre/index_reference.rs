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
const DEFAULT_INDEX_REFERENCE_THREADS: u32 = 2;

/// # Errors
/// Returns an error if any requested reference-indexing tool is not admitted for the stage.
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

/// # Errors
/// Returns an error if the requested reference-indexing tool cannot be planned.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    reference_fasta: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_with_options(tool, reference_fasta, out_dir, &IndexReferencePlanOptions::default())
}

/// # Errors
/// Returns an error if the requested reference-indexing tool or options are unsupported, or if
/// the stage plan cannot be built.
pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    reference_fasta: &Path,
    out_dir: &Path,
    options: &IndexReferencePlanOptions,
) -> Result<StagePlanV1> {
    let output = reference_index_output(&tool.tool_id.0, out_dir)?;
    let report_json = out_dir.join("index_reference_report.json");
    let threads = options.threads.unwrap_or(DEFAULT_INDEX_REFERENCE_THREADS).max(1);
    let mut resources = tool.resources.clone();
    resources.threads = threads;
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
            template: index_reference_command(&tool.tool_id.0, reference_fasta, &output, threads)?,
        },
        resources,
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reference_fasta"),
                reference_fasta.to_path_buf(),
                ArtifactRole::Reference,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("reference_index"),
                    output.clone(),
                    ArtifactRole::Index,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("report_json"),
                    report_json.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
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
        _ => None,
    }
}

fn reference_index_output(tool_id: &str, out_dir: &Path) -> Result<std::path::PathBuf> {
    match tool_id {
        "bowtie2_build" => Ok(out_dir.join("reference_index").join("bowtie2").join("reference")),
        "star" => Ok(out_dir.join("reference_index").join("star")),
        _ => Err(anyhow!("unsupported reference indexing tool for stage planning: {tool_id}")),
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
            "sh".to_string(),
            "-lc".to_string(),
            format!(
                "set -eu\nmkdir -p {}\nbowtie2-build --threads {} {} {}",
                shell_quote_path(
                    output
                        .parent()
                        .ok_or_else(|| anyhow!("index output has no parent: {}", output.display()))?
                ),
                threads,
                shell_quote_path(reference_fasta),
                shell_quote_path(output),
            ),
        ]),
        "star" => Ok(vec![
            "sh".to_string(),
            "-lc".to_string(),
            format!(
                "set -eu\nmkdir -p {}\nSTAR --runMode genomeGenerate --runThreadN {} --genomeDir {} --genomeFastaFiles {}",
                shell_quote_path(output),
                threads,
                shell_quote_path(output),
                shell_quote_path(reference_fasta),
            ),
        ]),
        _ => Err(anyhow!(
            "unsupported reference indexing tool for stage planning: {tool_id}"
        )),
    }
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1,
    };

    fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: tool_id.try_into().expect("tool id"),
            tool_version: "test".to_string(),
            image: ContainerImageRefV1 { image: "tool".to_string(), digest: None },
            command: CommandSpecV1 { template: vec![] },
            resources: ToolConstraints {
                runtime: "local".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        }
    }

    #[test]
    fn bowtie2_build_command_creates_parent_directory() {
        let plan = plan_with_options(
            &tool("bowtie2_build"),
            Path::new("reference.fa"),
            Path::new("out"),
            &IndexReferencePlanOptions { threads: Some(4) },
        )
        .expect("plan");
        assert_eq!(plan.command.template[0], "sh");
        assert!(plan.command.template[2].contains("mkdir -p 'out/reference_index/bowtie2'"));
        assert!(plan.command.template[2].contains(
            "bowtie2-build --threads 4 'reference.fa' 'out/reference_index/bowtie2/reference'"
        ));
    }

    #[test]
    fn star_command_creates_genome_dir() {
        let plan = plan_with_options(
            &tool("star"),
            Path::new("reference.fa"),
            Path::new("out"),
            &IndexReferencePlanOptions { threads: Some(6) },
        )
        .expect("plan");
        assert_eq!(plan.command.template[0], "sh");
        assert!(plan.command.template[2].contains("mkdir -p 'out/reference_index/star'"));
        assert!(plan.command.template[2].contains("STAR --runMode genomeGenerate --runThreadN 6 --genomeDir 'out/reference_index/star' --genomeFastaFiles 'reference.fa'"));
    }
}
