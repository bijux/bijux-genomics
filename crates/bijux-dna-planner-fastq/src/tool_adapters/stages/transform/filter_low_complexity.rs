use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::STAGE_FILTER_LOW_COMPLEXITY;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_FILTER_LOW_COMPLEXITY;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, Default)]
pub struct LowComplexityPlanOptions {
    pub entropy_threshold: Option<f64>,
    pub polyx_threshold: Option<u32>,
}

pub fn normalize_low_complexity_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

fn low_complexity_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "dustmasker" => Some("dustmasker.fastq.gz"),
        "prinseq" => Some("prinseq_good.fastq"),
        "bbduk" => Some("bbduk.fastq.gz"),
        _ => None,
    }
}

/// Build a low-complexity filter plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_low_complexity(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &LowComplexityPlanOptions,
) -> Result<StagePlanV1> {
    let output_name = low_complexity_output_name(&tool.tool_id.0)
        .ok_or_else(|| anyhow!("unsupported low-complexity tool"))?;
    let output_r1 = if r2.is_some() {
        out_dir.join(format!("R1.{output_name}"))
    } else {
        out_dir.join(output_name)
    };
    let output_r2 = r2.map(|_| out_dir.join(format!("R2.{output_name}")));
    let report = out_dir.join("low_complexity_report.json");
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
    let mut outputs = vec![ArtifactRef::required(
        if output_r2.is_some() {
            ArtifactId::from_static("filtered_fastq_r1")
        } else {
            ArtifactId::from_static("filtered_fastq")
        },
        output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("filtered_fastq_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("filter_report_json"),
        report.clone(),
        ArtifactRole::ReportJson,
    ));
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 {
            template: tool.command.template.to_vec(),
        },
        resources: tool.resources.clone(),
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report,
            "entropy_threshold": options.entropy_threshold,
            "polyx_threshold": options.polyx_threshold,
        }),
        effective_params: serde_json::json!({
            "paired_mode": if r2.is_some() { "paired_end" } else { "single_end" },
            "threads": tool.resources.threads,
            "entropy_threshold": options.entropy_threshold,
            "polyx_threshold": options.polyx_threshold,
        }),
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}
