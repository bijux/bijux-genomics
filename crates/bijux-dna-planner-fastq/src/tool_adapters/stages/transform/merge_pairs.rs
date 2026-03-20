use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{merge::MergeEffectiveParams, PairedMode};
use bijux_dna_domain_fastq::STAGE_MERGE_PAIRS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_MERGE_PAIRS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_merge_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

/// Build a merge plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_merge(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let outputs = merge_outputs(&tool.tool_id.0, out_dir)?;
    let effective_params = MergeEffectiveParams {
        paired_mode: PairedMode::PairedEnd,
        threads: tool.resources.threads,
        merge_overlap: None,
        min_len: None,
    };
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 {
            template: merge_command_template(&tool.tool_id.0, r1, r2, out_dir, &outputs, tool)?,
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("reads_r1"),
                    r1.to_path_buf(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("reads_r2"),
                    r2.to_path_buf(),
                    ArtifactRole::Reads,
                ),
            ],
            outputs: merge_artifacts(&outputs),
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "r1": r1,
            "r2": r2,
            "merged_reads": outputs.merged_reads,
            "unmerged_reads_r1": outputs.unmerged_reads_r1,
            "unmerged_reads_r2": outputs.unmerged_reads_r2,
            "report_json": outputs.report_json
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize merge effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

#[derive(Debug)]
struct MergeOutputs {
    merged_reads: std::path::PathBuf,
    unmerged_reads_r1: Option<std::path::PathBuf>,
    unmerged_reads_r2: Option<std::path::PathBuf>,
    report_json: std::path::PathBuf,
}

fn merge_outputs(tool: &str, out_dir: &Path) -> Result<MergeOutputs> {
    let report_json = out_dir.join("merge_report.json");
    let outputs = match tool {
        "pear" => {
            let prefix = out_dir.join("pear");
            MergeOutputs {
                merged_reads: prefix.with_extension("assembled.fastq"),
                unmerged_reads_r1: Some(out_dir.join("pear.unassembled.forward.fastq")),
                unmerged_reads_r2: Some(out_dir.join("pear.unassembled.reverse.fastq")),
                report_json,
            }
        }
        "vsearch" => MergeOutputs {
            merged_reads: out_dir.join("vsearch.merged.fastq"),
            unmerged_reads_r1: Some(out_dir.join("vsearch.unmerged_r1.fastq")),
            unmerged_reads_r2: Some(out_dir.join("vsearch.unmerged_r2.fastq")),
            report_json,
        },
        "bbmerge" => MergeOutputs {
            merged_reads: out_dir.join("bbmerge.merged.fastq"),
            unmerged_reads_r1: Some(out_dir.join("bbmerge.unmerged_r1.fastq")),
            unmerged_reads_r2: Some(out_dir.join("bbmerge.unmerged_r2.fastq")),
            report_json,
        },
        "flash2" => MergeOutputs {
            merged_reads: out_dir.join("flash2.extendedFrags.fastq"),
            unmerged_reads_r1: Some(out_dir.join("flash2.notCombined_1.fastq")),
            unmerged_reads_r2: Some(out_dir.join("flash2.notCombined_2.fastq")),
            report_json,
        },
        "leehom" => MergeOutputs {
            merged_reads: out_dir.join("leehom.fastq.gz"),
            unmerged_reads_r1: None,
            unmerged_reads_r2: None,
            report_json,
        },
        _ => return Err(anyhow!("unsupported merge tool")),
    };
    Ok(outputs)
}

fn merge_artifacts(outputs: &MergeOutputs) -> Vec<ArtifactRef> {
    let mut artifacts = vec![ArtifactRef::required(
        ArtifactId::from_static("merged_reads"),
        outputs.merged_reads.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(unmerged_reads_r1) = &outputs.unmerged_reads_r1 {
        artifacts.push(ArtifactRef::optional(
            ArtifactId::from_static("unmerged_reads_r1"),
            unmerged_reads_r1.clone(),
            ArtifactRole::Reads,
        ));
    }
    if let Some(unmerged_reads_r2) = &outputs.unmerged_reads_r2 {
        artifacts.push(ArtifactRef::optional(
            ArtifactId::from_static("unmerged_reads_r2"),
            unmerged_reads_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    artifacts.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        outputs.report_json.clone(),
        ArtifactRole::MetricsJson,
    ));
    artifacts
}

fn merge_command_template(
    tool: &str,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    outputs: &MergeOutputs,
    tool_spec: &ToolExecutionSpecV1,
) -> Result<Vec<String>> {
    let command = match tool {
        "pear" => {
            let prefix = out_dir.join("pear");
            vec![
                "pear".to_string(),
                "-f".to_string(),
                r1.display().to_string(),
                "-r".to_string(),
                r2.display().to_string(),
                "-o".to_string(),
                prefix.display().to_string(),
            ]
        }
        "vsearch" => vec![
            "vsearch".to_string(),
            "--fastq_mergepairs".to_string(),
            r1.display().to_string(),
            "--reverse".to_string(),
            r2.display().to_string(),
            "--fastqout".to_string(),
            outputs.merged_reads.display().to_string(),
            "--fastqout_notmerged_fwd".to_string(),
            outputs
                .unmerged_reads_r1
                .as_ref()
                .ok_or_else(|| anyhow!("vsearch merge requires unmerged_reads_r1 output"))?
                .display()
                .to_string(),
            "--fastqout_notmerged_rev".to_string(),
            outputs
                .unmerged_reads_r2
                .as_ref()
                .ok_or_else(|| anyhow!("vsearch merge requires unmerged_reads_r2 output"))?
                .display()
                .to_string(),
        ],
        "bbmerge" => vec![
            "bbmerge".to_string(),
            format!("in1={}", r1.display()),
            format!("in2={}", r2.display()),
            format!("out={}", outputs.merged_reads.display()),
            format!(
                "outu1={}",
                outputs
                    .unmerged_reads_r1
                    .as_ref()
                    .ok_or_else(|| anyhow!("bbmerge merge requires unmerged_reads_r1 output"))?
                    .display()
            ),
            format!(
                "outu2={}",
                outputs
                    .unmerged_reads_r2
                    .as_ref()
                    .ok_or_else(|| anyhow!("bbmerge merge requires unmerged_reads_r2 output"))?
                    .display()
            ),
        ],
        "flash2" => vec![
            "flash2".to_string(),
            "-o".to_string(),
            "flash2".to_string(),
            "-d".to_string(),
            out_dir.display().to_string(),
            r1.display().to_string(),
            r2.display().to_string(),
        ],
        "leehom" => tool_spec.command.template.to_vec(),
        _ => return Err(anyhow!("unsupported merge tool")),
    };
    Ok(command)
}

fn normalize_tools_with_allowlist(
    tools: &[String],
    allowlist: &[bijux_dna_core::ids::ToolId],
) -> Result<Vec<String>> {
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
