use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
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
pub type DetectAdaptersPlanOptions = crate::DetectAdaptersStageParams;

/// # Errors
/// Returns an error if adapter detection cannot be planned for the requested tool.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_with_options(tool, r1, r2, out_dir, &DetectAdaptersPlanOptions::default())
}

/// # Errors
/// Returns an error if adapter detection cannot be planned for the requested tool or options.
pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &DetectAdaptersPlanOptions,
) -> Result<StagePlanV1> {
    let report = out_dir.join("adapter_report.json");
    let adapter_evidence_dir = out_dir.join("fastqc");
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let effective_params = DetectAdaptersEffectiveParams {
        schema_version: DETECT_ADAPTERS_SCHEMA_VERSION.to_string(),
        paired_mode: if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: effective_threads,
        sample_reads: None,
        inspection_mode: AdapterInspectionMode::EvidenceOnly,
        report_only: true,
        evidence_engine: tool.tool_id.to_string(),
        evidence_scope: AdapterEvidenceScope::FullInput,
        evidence_format: AdapterEvidenceFormat::FastqcSummary,
        evidence_artifact_id: "report_json".to_string(),
    };
    let command_template =
        detect_adapters_command(&tool.tool_id.0, r1, r2, &adapter_evidence_dir, effective_threads)?;
    let mut resources = tool.resources.clone();
    resources.threads = effective_threads;
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
        command: CommandSpecV1 { template: command_template },
        resources,
        io: StageIO {
            inputs,
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("report_json"),
                    report.clone(),
                    ArtifactRole::ReportJson,
                ),
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
            "threads": effective_threads,
            "report_json": report,
            "adapter_evidence_dir": adapter_evidence_dir,
            "sample_reads": effective_params.sample_reads,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize detect adapters effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
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
            Ok(wrap_fastqc_command(&command, adapter_evidence_dir))
        }
        _ => Err(anyhow!("unsupported adapter detection tool for stage planning: {tool_id}")),
    }
}

fn wrap_fastqc_command(command: &[String], output_dir: &Path) -> Vec<String> {
    vec![
        "sh".to_string(),
        "-lc".to_string(),
        format!("mkdir -p {}\n{}", shell_quote(output_dir), shell_join(command)),
    ]
}

fn shell_join(command: &[String]) -> String {
    command.iter().map(|part| shell_quote_str(part)).collect::<Vec<_>>().join(" ")
}

fn shell_quote(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::plan;
    use anyhow::Result;
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
    };

    #[test]
    fn detect_adapters_plan_emits_canonical_report_and_full_input_scope() -> Result<()> {
        let temp = std::env::temp_dir().join("bijux-detect-adapters-plan-test");
        bijux_dna_infra::ensure_dir(&temp)?;
        let tool = ToolExecutionSpecV1 {
            tool_id: ToolId::from_static("fastqc"),
            tool_version: "0.12.1".to_string(),
            image: ContainerImageRefV1 {
                image: "bijuxdna/fastqc".to_string(),
                digest: Some("sha256:test".to_string()),
            },
            command: CommandSpecV1 {
                template: vec!["fastqc".to_string(), "{{reads_r1}}".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 4,
            },
        };
        let r1 = temp.join("reads_R1.fastq.gz");
        let r2 = temp.join("reads_R2.fastq.gz");
        let out_dir = temp.join("out");
        let plan = plan(&tool, &r1, Some(&r2), &out_dir)?;

        assert!(plan.io.outputs.iter().any(|artifact| artifact.name.as_str() == "report_json"));
        assert_eq!(plan.effective_params["evidence_scope"], "full_input");
        assert_eq!(plan.effective_params["sample_reads"], serde_json::Value::Null);
        assert_eq!(plan.effective_params["evidence_artifact_id"], "report_json");
        assert_eq!(plan.command.template[0], "sh");
        assert_eq!(plan.command.template[1], "-lc");
        assert!(plan.command.template[2].contains("mkdir -p"));
        assert!(plan.command.template[2].contains("fastqc"));
        Ok(())
    }
}
