use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::trim::{
    TrimTerminalDamageParams, TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_TERMINAL_DAMAGE;
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_TRIM_TERMINAL_DAMAGE;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub type TrimTerminalDamagePlanOptions = crate::TrimTerminalDamageStageParams;

fn output_name(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "cutadapt" => Some("trim_terminal_damage.cutadapt.fastq.gz"),
        "seqkit" => Some("trim_terminal_damage.seqkit.fastq.gz"),
        _ => None,
    }
}

/// # Errors
/// Returns an error when the tool does not support `fastq.trim_terminal_damage`.
pub fn plan_trim_terminal_damage(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    damage_mode: &str,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
) -> Result<StagePlanV1> {
    plan_trim_terminal_damage_with_options(
        tool,
        r1,
        r2,
        out_dir,
        &TrimTerminalDamagePlanOptions {
            damage_mode: damage_mode.to_string(),
            trim_5p_bases,
            trim_3p_bases,
        },
    )
}

/// # Errors
/// Returns an error when the tool does not support `fastq.trim_terminal_damage`.
pub fn plan_trim_terminal_damage_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &TrimTerminalDamagePlanOptions,
) -> Result<StagePlanV1> {
    let out_name = output_name(tool.tool_id.as_str())
        .ok_or_else(|| anyhow!("unsupported trim_terminal_damage tool {}", tool.tool_id))?;
    let output_r1 = if r2.is_some() {
        out_dir.join(format!("R1.{out_name}"))
    } else {
        out_dir.join(out_name)
    };
    let output_r2 = r2.map(|_| out_dir.join(format!("R2.{out_name}")));
    let report = out_dir.join("trim_terminal_damage_report.json");
    let command_template = trim_terminal_damage_command(
        &tool.tool_id.0,
        r1,
        r2,
        &output_r1,
        output_r2.as_deref(),
        &report,
        &options.damage_mode,
        options.trim_5p_bases,
        options.trim_3p_bases,
    )?;
    let effective_params = TrimTerminalDamageParams {
        schema_version: TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::from_has_r2(r2.is_some()),
        threads: tool.resources.threads,
        damage_mode: options.damage_mode.clone(),
        trim_5p_bases: options.trim_5p_bases,
        trim_3p_bases: options.trim_3p_bases,
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
        report.clone(),
        ArtifactRole::ReportJson,
    ));
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
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report,
        }),
        effective_params: serde_json::to_value(&effective_params)?,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "damage-aware terminal trimming"),
    })
}

fn trim_terminal_damage_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report: &Path,
    damage_mode: &str,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
) -> Result<Vec<String>> {
    match tool_id {
        "cutadapt" => {
            let mut command = vec![
                "cutadapt".to_string(),
                "-u".to_string(),
                trim_5p_bases.to_string(),
                "-u".to_string(),
                format!("-{trim_3p_bases}"),
                "--json".to_string(),
                report.display().to_string(),
                "-o".to_string(),
                output_r1.display().to_string(),
            ];
            if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
                command.push("-p".to_string());
                command.push(output_r2.display().to_string());
                command.push(r1.display().to_string());
                command.push(r2.display().to_string());
            } else {
                command.push(r1.display().to_string());
            }
            Ok(command)
        }
        "seqkit" => {
            let region = terminal_trim_region(trim_5p_bases, trim_3p_bases);
            let mut script = format!(
                "set -euo pipefail\nseqkit subseq -r {} {} -o {}\n",
                shell_quote_str(&region),
                shell_quote_path(r1),
                shell_quote_path(output_r1),
            );
            if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
                script.push_str(&format!(
                    "seqkit subseq -r {} {} -o {}\n",
                    shell_quote_str(&region),
                    shell_quote_path(r2),
                    shell_quote_path(output_r2),
                ));
            }
            let report_payload = serde_json::json!({
                "schema_version": "bijux.fastq.trim_terminal_damage.report.v1",
                "tool_id": tool_id,
                "damage_mode": damage_mode,
                "trim_5p_bases": trim_5p_bases,
                "trim_3p_bases": trim_3p_bases,
                "input_r1": r1,
                "input_r2": r2,
                "output_r1": output_r1,
                "output_r2": output_r2,
            });
            script.push_str(&format!(
                "printf '%s\\n' {} > {}\n",
                shell_quote_str(&report_payload.to_string()),
                shell_quote_path(report),
            ));
            Ok(vec!["sh".to_string(), "-lc".to_string(), script])
        }
        _ => Err(anyhow!(
            "unsupported trim_terminal_damage tool for stage planning: {tool_id}"
        )),
    }
}

fn terminal_trim_region(trim_5p_bases: u32, trim_3p_bases: u32) -> String {
    let start = trim_5p_bases.saturating_add(1);
    let end = if trim_3p_bases == 0 {
        "-1".to_string()
    } else {
        format!("-{}", trim_3p_bases.saturating_add(1))
    };
    format!("{start}:{end}")
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
