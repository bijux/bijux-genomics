use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{
    ArtifactRef, ContainerImageRefV1, StageIO, StageId, StagePlanV1, StageVersion,
    ToolExecutionSpecV1,
};
use bijux_domain_fastq::params::{qc_post::QcPostEffectiveParams, PairedMode};

pub const STAGE_ID: &str = "fastq.qc_post";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_qc_post_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["fastqc", "multiqc"];
    normalize_tools_with_allowlist(tools, &allowed)
}

#[must_use]
pub fn aux_tool_ids() -> &'static [&'static str] {
    &["fastqc"]
}

/// Build a qc_post plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_qc_post(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    out_dir: &Path,
    aux_images: std::collections::BTreeMap<String, ContainerImageRefV1>,
    raw_r1: Option<&Path>,
) -> Result<StagePlanV1> {
    if normalize_qc_post_tool_list(std::slice::from_ref(&tool.tool_id.0))?.is_empty() {
        return Err(anyhow!("unsupported qc_post tool"));
    }
    let mut params = serde_json::json!({
        "tool": tool.tool_id.0,
        "input": r1,
        "out_dir": out_dir
    });
    if let Some(raw) = raw_r1 {
        params["raw_r1"] = serde_json::json!(raw);
    }
    let effective_params = QcPostEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: tool.resources.threads,
    };
    let outputs = if tool.tool_id.0 == "multiqc" {
        vec![
            ArtifactRef {
                name: "multiqc_report".to_string(),
                path: out_dir.join("multiqc_report.html"),
            },
            ArtifactRef {
                name: "multiqc_data".to_string(),
                path: out_dir.join("multiqc_data"),
            },
        ]
    } else {
        Vec::new()
    };
    Ok(StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: r1.to_path_buf(),
            }],
            outputs,
        },
        out_dir: out_dir.to_path_buf(),
        params,
        effective_params: serde_json::to_value(&effective_params)
            .expect("serialize qc_post effective params"),
        aux_images,
        reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
    })
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
