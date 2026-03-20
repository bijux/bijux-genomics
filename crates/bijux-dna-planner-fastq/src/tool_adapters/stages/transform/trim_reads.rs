use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{trim::TrimEffectiveParams, PairedMode};
use bijux_dna_domain_fastq::STAGE_TRIM_READS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_TRIM_READS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone)]
pub struct TrimUserConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub r2: Option<std::path::PathBuf>,
    pub out_dir: std::path::PathBuf,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct TrimEffectiveConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub r2: Option<std::path::PathBuf>,
    pub out_dir: std::path::PathBuf,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
}

pub fn trim_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fastp" => Some("fastp.fastq.gz"),
        "cutadapt" => Some("cutadapt.fastq.gz"),
        "atropos" => Some("atropos.fastq.gz"),
        "bbduk" => Some("bbduk.fastq.gz"),
        "adapterremoval" => Some("adapterremoval.fastq.gz"),
        "trimmomatic" => Some("trimmomatic.fastq.gz"),
        "trim_galore" => Some("trimmed_trimmed.fq.gz"),
        "prinseq" => Some("prinseq_good.fastq"),
        "seqkit" => Some("seqkit.fastq.gz"),
        "skewer" => Some("skewer.fastq.gz"),
        "leehom" => Some("leehom.fastq.gz"),
        "alientrimmer" => Some("alientrimmer.fastq.gz"),
        "fastx_clipper" => Some("fastx_clipper.fastq.gz"),
        _ => None,
    }
}

pub fn resolve_config(user: TrimUserConfig) -> TrimEffectiveConfig {
    TrimEffectiveConfig {
        tool: user.tool,
        r1: user.r1,
        r2: user.r2,
        out_dir: user.out_dir,
        adapter_bank: user.adapter_bank,
        polyx_bank: user.polyx_bank,
        contaminant_bank: user.contaminant_bank,
    }
}

/// Build a trim command plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<StagePlanV1> {
    let output_name =
        trim_output_name(&tool.tool_id.0).ok_or_else(|| anyhow!("unsupported trim tool"))?;
    let output_r1 = if r2.is_some() {
        out_dir.join(format!("R1.{output_name}"))
    } else {
        out_dir.join(output_name)
    };
    let output_r2 = r2.map(|_| out_dir.join(format!("R2.{output_name}")));
    let mut params = serde_json::json!({
        "tool": tool.tool_id.0,
        "input_r1": r1,
        "input_r2": r2,
        "output_r1": output_r1,
        "output_r2": output_r2
    });
    if let Some(adapter_bank) = adapter_bank {
        if let Some(map) = params.as_object_mut() {
            map.insert("adapter_bank".to_string(), adapter_bank.clone());
        }
    }
    if let Some(polyx_bank) = polyx_bank {
        if let Some(map) = params.as_object_mut() {
            map.insert("polyx_bank".to_string(), polyx_bank.clone());
        }
    }
    if let Some(contaminant_bank) = contaminant_bank {
        if let Some(map) = params.as_object_mut() {
            map.insert("contaminant_bank".to_string(), contaminant_bank.clone());
        }
    }
    let effective_params = TrimEffectiveParams {
        paired_mode: if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: tool.resources.threads,
        min_len: 0,
        q_cutoff: None,
        adapter_policy: if adapter_bank.is_some() {
            "bank".to_string()
        } else {
            "none".to_string()
        },
        damage_mode: None,
        polyx_policy: polyx_bank.as_ref().map(|_| "bank".to_string()),
        n_policy: None,
        contaminant_policy: contaminant_bank.as_ref().map(|_| "bank".to_string()),
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
        ArtifactId::from_static("trimmed_reads_r1"),
        output_r1.clone(),
        ArtifactRole::TrimmedReads,
    )];
    if let Some(output_r2) = &output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("trimmed_reads_r2"),
            output_r2.clone(),
            ArtifactRole::TrimmedReads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        out_dir.join("trim_report.json"),
        ArtifactRole::ReportJson,
    ));
    let report_json = out_dir.join("trim_report.json");
    let command_template =
        trim_command_template(tool, r1, r2, &output_r1, output_r2.as_deref(), &report_json)?;
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
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params,
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize trim effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

/// Build a trim plan from resolved config.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_from_config(
    tool: &ToolExecutionSpecV1,
    config: &TrimEffectiveConfig,
) -> Result<StagePlanV1> {
    plan(
        tool,
        &config.r1,
        config.r2.as_deref(),
        &config.out_dir,
        config.adapter_bank.as_ref(),
        config.polyx_bank.as_ref(),
        config.contaminant_bank.as_ref(),
    )
}

fn trim_command_template(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
) -> Result<Vec<String>> {
    if tool.tool_id.as_str() == "fastp" {
        let mut command = vec![
            "fastp".to_string(),
            "--in1".to_string(),
            r1.display().to_string(),
            "--out1".to_string(),
            output_r1.display().to_string(),
            "--json".to_string(),
            report_json.display().to_string(),
            "--thread".to_string(),
            tool.resources.threads.to_string(),
        ];
        if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
            command.extend([
                "--in2".to_string(),
                r2.display().to_string(),
                "--out2".to_string(),
                output_r2.display().to_string(),
                "--detect_adapter_for_pe".to_string(),
            ]);
        }
        return Ok(command);
    }
    if tool.tool_id.as_str() == "trim_galore" {
        return trim_galore_command_template(r1, r2, output_r1, output_r2, report_json);
    }
    let rendered = crate::tool_adapters::template_render::render_command_template(
        &tool.command.template,
        &[
            ("reads", Some(r1.display().to_string())),
            ("reads_r1", Some(r1.display().to_string())),
            ("reads_r2", r2.map(|path| path.display().to_string())),
            ("trimmed_reads", Some(output_r1.display().to_string())),
            (
                "trimmed_reads_dir",
                output_r1.parent().map(|path| path.display().to_string()),
            ),
            ("trimmed_reads_r1", Some(output_r1.display().to_string())),
            (
                "trimmed_reads_r2",
                output_r2.map(|path| path.display().to_string()),
            ),
            ("report_json", Some(report_json.display().to_string())),
        ],
    )?;
    Ok(wrap_trim_command_with_report(
        &tool.tool_id.0,
        rendered,
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
    ))
}

fn trim_galore_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
) -> Result<Vec<String>> {
    let output_dir = output_r1
        .parent()
        .ok_or_else(|| anyhow!("trim_galore output path must have a parent directory"))?;
    let working_dir = output_dir.join("trim_galore_run");
    let mut script = format!(
        "set -euo pipefail\nmkdir -p {}\ntrim_galore --output_dir {}",
        shell_quote_path(&working_dir),
        shell_quote_path(&working_dir),
    );
    if r2.is_some() {
        script.push_str(" --paired");
    }
    script.push(' ');
    script.push_str(&shell_quote_path(r1));
    if let Some(r2) = r2 {
        script.push(' ');
        script.push_str(&shell_quote_path(r2));
    }
    script.push('\n');
    script.push_str(&format!(
        "mv {} {}\n",
        shell_quote_path(&trim_galore_output_path(&working_dir, r1)),
        shell_quote_path(output_r1),
    ));
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        script.push_str(&format!(
            "mv {} {}\n",
            shell_quote_path(&trim_galore_output_path(&working_dir, r2)),
            shell_quote_path(output_r2),
        ));
    }
    script.push_str(&write_trim_report_script(
        "trim_galore",
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
    ));
    Ok(vec!["sh".to_string(), "-lc".to_string(), script])
}

fn trim_galore_output_path(output_dir: &Path, reads: &Path) -> PathBuf {
    let file_name = reads
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("reads.fastq.gz");
    let trimmed_name = if let Some(stripped) = file_name.strip_suffix(".fastq.gz") {
        format!("{stripped}_trimmed.fq.gz")
    } else if let Some(stripped) = file_name.strip_suffix(".fq.gz") {
        format!("{stripped}_trimmed.fq.gz")
    } else if let Some(stripped) = file_name.strip_suffix(".fastq") {
        format!("{stripped}_trimmed.fq")
    } else if let Some(stripped) = file_name.strip_suffix(".fq") {
        format!("{stripped}_trimmed.fq")
    } else {
        format!("{file_name}_trimmed.fq.gz")
    };
    output_dir.join(trimmed_name)
}

fn wrap_trim_command_with_report(
    tool_id: &str,
    command: Vec<String>,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
) -> Vec<String> {
    let mut script = format!("set -euo pipefail\n{}\n", shell_join(&command));
    script.push_str(&write_trim_report_script(
        tool_id, r1, r2, output_r1, output_r2, report_json,
    ));
    vec!["sh".to_string(), "-lc".to_string(), script]
}

fn write_trim_report_script(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
) -> String {
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.trim_reads.report.v1",
        "stage_id": STAGE_ID.as_str(),
        "tool_id": tool_id,
        "input_r1": r1,
        "input_r2": r2,
        "output_r1": output_r1,
        "output_r2": output_r2,
    });
    format!(
        "printf '%s\\n' {} > {}\n",
        shell_quote_str(&payload.to_string()),
        shell_quote_path(report_json),
    )
}

fn shell_join(command: &[String]) -> String {
    command
        .iter()
        .map(|part| shell_quote_str(part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
