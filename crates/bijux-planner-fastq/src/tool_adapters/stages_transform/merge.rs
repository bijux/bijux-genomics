use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{ArtifactRef, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_fastq::params::{merge::MergeEffectiveParams, PairedMode};

pub const STAGE_ID: &str = "fastq.merge";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_merge_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["pear", "vsearch", "bbmerge", "flash2"];
    normalize_tools_with_allowlist(tools, &allowed)
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
    let output_name =
        merge_output_name(&tool.tool_id.0).ok_or_else(|| anyhow!("unsupported merge tool"))?;
    let output = out_dir.join(output_name);
    let effective_params = MergeEffectiveParams {
        paired_mode: PairedMode::PairedEnd,
        threads: tool.resources.threads,
        merge_overlap: None,
        min_len: None,
    };
    Ok(StagePlanV1 {
        stage_id: StageId::from_static(STAGE_ID),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![
                ArtifactRef {
                    name: "reads_r1".to_string(),
                    path: r1.to_path_buf(),
                },
                ArtifactRef {
                    name: "reads_r2".to_string(),
                    path: r2.to_path_buf(),
                },
            ],
            outputs: vec![ArtifactRef {
                name: "merged_reads".to_string(),
                path: output.clone(),
            }],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "r1": r1,
            "r2": r2,
            "output": output
        }),
        effective_params: serde_json::to_value(&effective_params)
            .expect("serialize merge effective params"),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
    })
}

fn merge_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "pear" => Some("pear.fastq.gz"),
        "vsearch" => Some("vsearch.fastq.gz"),
        "bbmerge" => Some("bbmerge.fastq.gz"),
        "flash2" => Some("flash2.fastq.gz"),
        _ => None,
    }
}

fn normalize_tools_with_allowlist(tools: &[String], allowlist: &[&str]) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    for tool in &normalized {
        if !allowlist.contains(&tool.as_str()) {
            return Err(anyhow!("unsupported tool {tool}"));
        }
    }
    Ok(normalized)
}
