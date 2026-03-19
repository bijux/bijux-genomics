use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{screen::RrnaEffectiveParams, PairedMode};
use bijux_dna_domain_fastq::STAGE_DEPLETE_RRNA;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_DEPLETE_RRNA;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_rrna_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

/// Build an rRNA screening plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_rrna(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    normalize_rrna_tool_list(std::slice::from_ref(&tool_id))?;
    let filtered_reads_r1 = if r2.is_some() {
        out_dir.join("rrna_filtered_R1.fastq.gz")
    } else {
        out_dir.join("rrna_filtered.fastq.gz")
    };
    let filtered_reads_r2 = r2.map(|_| out_dir.join("rrna_filtered_R2.fastq.gz"));
    let report = out_dir.join("rrna_report.tsv");
    let metrics = out_dir.join("rrna_report.json");
    let effective_params = RrnaEffectiveParams {
        paired_mode: if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: tool.resources.threads,
        contaminant_db: Some("rrna_reference".to_string()),
    };
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
        if filtered_reads_r2.is_some() {
            ArtifactId::from_static("rrna_filtered_reads_r1")
        } else {
            ArtifactId::from_static("rrna_filtered_reads")
        },
        filtered_reads_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(filtered_reads_r2) = &filtered_reads_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("rrna_filtered_reads_r2"),
            filtered_reads_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("rrna_report_tsv"),
        report.clone(),
        ArtifactRole::SummaryTsv,
    ));
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("rrna_report_json"),
        metrics.clone(),
        ArtifactRole::MetricsJson,
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
            "filtered_reads_r1": filtered_reads_r1,
            "filtered_reads_r2": filtered_reads_r2,
            "report_tsv": report,
            "report_json": metrics
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize rrna effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}
