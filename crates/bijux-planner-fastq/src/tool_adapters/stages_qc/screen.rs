use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{ArtifactRef, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};
use bijux_domain_fastq::params::{screen::ScreenEffectiveParams, PairedMode};

pub const STAGE_ID: &str = "fastq.screen";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "kraken2",
        "centrifuge",
        "metaphlan",
        "kaiju",
        "fastq_screen",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

/// Build a screen plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_screen(tool: &ToolExecutionSpecV1, r1: &Path, out_dir: &Path) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_screen_tool_list(std::slice::from_ref(&tool_id))?;
    let report = out_dir.join("screen_report.tsv");
    let effective_params = ScreenEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: tool.resources.threads,
        contaminant_db: None,
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
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: r1.to_path_buf(),
            }],
            outputs: vec![ArtifactRef {
                name: "screen_report_tsv".to_string(),
                path: report.clone(),
            }],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input": r1,
            "out_dir": out_dir,
            "report": report
        }),
        effective_params: serde_json::to_value(&effective_params)
            .expect("serialize screen effective params"),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
    })
}

fn normalize_tools_with_allowlist(tools: &[String], allowlist: &[&str]) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.contains(&tool.as_str()) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}
