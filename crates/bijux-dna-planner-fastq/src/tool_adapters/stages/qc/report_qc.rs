use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, StageId,
    StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    qc_post::{
        QcAggregationEngine, QcAggregationScope, QcPostEffectiveParams, REPORT_QC_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::STAGE_REPORT_QC;
use bijux_dna_stage_contract::{StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_REPORT_QC;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_qc_post_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

#[must_use]
pub fn aux_tool_ids() -> &'static [&'static str] {
    &["fastqc"]
}

/// Build a QC reporting plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_qc_post(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    aux_images: std::collections::BTreeMap<String, ContainerImageRefV1>,
    raw_r1: Option<&Path>,
    raw_r2: Option<&Path>,
) -> Result<StagePlanV1> {
    let mut qc_inputs = vec![ArtifactRef::required(
        ArtifactId::from_static("reads_r1"),
        r1.to_path_buf(),
        ArtifactRole::Reads,
    )];
    if let Some(r2) = r2 {
        qc_inputs.push(ArtifactRef::required(
            ArtifactId::from_static("reads_r2"),
            r2.to_path_buf(),
            ArtifactRole::Reads,
        ));
    }
    plan_qc_post_with_qc_inputs(
        tool,
        &qc_inputs,
        out_dir,
        aux_images,
        if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        QcAggregationScope::FastqQcInputs,
        raw_r1,
        raw_r2,
    )
}

/// Build a QC reporting plan from governed upstream QC artifacts.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_qc_post_with_qc_inputs(
    tool: &ToolExecutionSpecV1,
    qc_inputs: &[ArtifactRef],
    out_dir: &Path,
    aux_images: std::collections::BTreeMap<String, ContainerImageRefV1>,
    paired_mode: PairedMode,
    aggregation_scope: QcAggregationScope,
    raw_r1: Option<&Path>,
    raw_r2: Option<&Path>,
) -> Result<StagePlanV1> {
    let tool_id = tool.tool_id.to_string();
    if normalize_qc_post_tool_list(std::slice::from_ref(&tool_id))?.is_empty() {
        return Err(anyhow!("unsupported report_qc tool"));
    }
    if qc_inputs.is_empty() {
        return Err(anyhow!(
            "fastq.report_qc requires governed QC artifacts and cannot aggregate raw FASTQ inputs"
        ));
    }
    let mut params = serde_json::json!({
        "tool": tool.tool_id.0,
        "qc_inputs": qc_inputs
            .iter()
            .map(|artifact| artifact.path.clone())
            .collect::<Vec<_>>(),
        "out_dir": out_dir
    });
    if let Some(raw) = raw_r1 {
        params["raw_r1"] = serde_json::json!(raw);
    }
    if let Some(raw) = raw_r2 {
        params["raw_r2"] = serde_json::json!(raw);
    }
    let effective_params = QcPostEffectiveParams {
        schema_version: REPORT_QC_SCHEMA_VERSION.to_string(),
        paired_mode,
        threads: tool.resources.threads,
        aggregation_engine: QcAggregationEngine::Multiqc,
        aggregation_scope,
    };
    let multiqc_data = out_dir.join("multiqc_data");
    let command_template = qc_post_command(&tool.tool_id.0, qc_inputs, &multiqc_data)?;
    let outputs = if tool.tool_id.0 == "multiqc" {
        vec![
            ArtifactRef::required(
                ArtifactId::from_static("multiqc_report"),
                out_dir.join("multiqc_report.html"),
                ArtifactRole::ReportHtml,
            ),
            ArtifactRef::required(
                ArtifactId::from_static("multiqc_data"),
                multiqc_data.clone(),
                ArtifactRole::Index,
            ),
        ]
    } else {
        Vec::new()
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
            template: command_template,
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: qc_inputs.to_vec(),
            outputs,
        },
        out_dir: out_dir.to_path_buf(),
        params,
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize report_qc effective params: {error}"))?,
        aux_images,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn qc_post_command(
    tool_id: &str,
    qc_inputs: &[ArtifactRef],
    multiqc_data: &Path,
) -> Result<Vec<String>> {
    match tool_id {
        "multiqc" => {
            let mut multiqc_inputs = qc_inputs.iter().map(qc_input_scan_path).collect::<Vec<_>>();
            multiqc_inputs.sort();
            multiqc_inputs.dedup();
            let mut command = vec![
                "multiqc".to_string(),
                "-o".to_string(),
                multiqc_data.display().to_string(),
                "-n".to_string(),
                "multiqc_report.html".to_string(),
            ];
            for input in multiqc_inputs {
                command.push(input.display().to_string());
            }
            Ok(command)
        }
        _ => Err(anyhow!("unsupported report_qc tool: {tool_id}")),
    }
}

fn qc_input_scan_path(artifact: &ArtifactRef) -> std::path::PathBuf {
    match artifact.path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => artifact.path.clone(),
    }
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
