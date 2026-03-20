use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{
    detect_adapters::{
        AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode,
        DetectAdaptersEffectiveParams, DETECT_ADAPTERS_SCHEMA_VERSION,
    },
    PairedMode,
};
use bijux_dna_domain_fastq::STAGE_DETECT_ADAPTERS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_DETECT_ADAPTERS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let report = out_dir.join("adapter_report.json");
    let adapter_evidence_dir = out_dir.join("fastqc");
    let effective_params = DetectAdaptersEffectiveParams {
        schema_version: DETECT_ADAPTERS_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: tool.resources.threads,
        sample_reads: Some(100_000),
        inspection_mode: AdapterInspectionMode::EvidenceOnly,
        report_only: true,
        evidence_engine: tool.tool_id.to_string(),
        evidence_scope: AdapterEvidenceScope::SampledReads,
        evidence_format: AdapterEvidenceFormat::FastqcSummary,
        evidence_artifact_id: "adapter_report".to_string(),
    };
    let command_template = detect_adapters_command(
        &tool.tool_id.0,
        r1,
        r2,
        &adapter_evidence_dir,
        tool.resources.threads,
    )?;
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
            template: command_template,
        },
        resources: tool.resources.clone(),
        io: StageIO {
            inputs,
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("adapter_report"),
                    report.clone(),
                    ArtifactRole::ReportJson,
                ),
                ArtifactRef::optional(
                    ArtifactId::from_static("adapter_evidence_dir"),
                    adapter_evidence_dir.clone(),
                    ArtifactRole::StageReport,
                ),
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "out_dir": out_dir,
            "report_json": report,
            "adapter_evidence_dir": adapter_evidence_dir,
            "sample_reads": effective_params.sample_reads,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize detect adapters effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn detect_adapters_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    adapter_evidence_dir: &Path,
    threads: u32,
) -> Result<Vec<String>> {
    match tool_id {
        "fastqc" => {
            let mut command = vec![
                "fastqc".to_string(),
                "--outdir".to_string(),
                adapter_evidence_dir.display().to_string(),
                "--threads".to_string(),
                threads.to_string(),
                r1.display().to_string(),
            ];
            if let Some(r2) = r2 {
                command.push(r2.display().to_string());
            }
            Ok(command)
        }
        _ => Err(anyhow!(
            "unsupported adapter detection tool for stage planning: {tool_id}"
        )),
    }
}
