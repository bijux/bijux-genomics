use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::trim::{TrimPolygTailsParams, TRIM_POLYG_TAILS_SCHEMA_VERSION};
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_POLYG_TAILS;
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_TRIM_POLYG_TAILS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimPolygPlanOptions {
    pub trim_polyg: bool,
    pub min_polyg_run: u32,
}

impl Default for TrimPolygPlanOptions {
    fn default() -> Self {
        Self {
            trim_polyg: true,
            min_polyg_run: 10,
        }
    }
}

fn output_name(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "fastp" => Some("polyg.fastp.fastq.gz"),
        "bbduk" => Some("polyg.bbduk.fastq.gz"),
        _ => None,
    }
}

/// # Errors
/// Returns an error when the tool does not support `fastq.trim_polyg_tails`.
pub fn plan_trim_polyg_tails(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    plan_trim_polyg_tails_with_options(tool, r1, r2, out_dir, &TrimPolygPlanOptions::default())
}

/// # Errors
/// Returns an error when the tool does not support `fastq.trim_polyg_tails`.
pub fn plan_trim_polyg_tails_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &TrimPolygPlanOptions,
) -> Result<StagePlanV1> {
    let out_name = output_name(tool.tool_id.as_str())
        .ok_or_else(|| anyhow!("unsupported trim_polyg_tails tool {}", tool.tool_id))?;
    let output_r1 = if r2.is_some() {
        out_dir.join(format!("R1.{out_name}"))
    } else {
        out_dir.join(out_name)
    };
    let output_r2 = r2.map(|_| out_dir.join(format!("R2.{out_name}")));
    let report = out_dir.join("trim_polyg_tails_report.json");
    let command_template = trim_polyg_command(
        &tool.tool_id.0,
        r1,
        r2,
        &output_r1,
        output_r2.as_deref(),
        &report,
        tool.resources.threads,
        options.trim_polyg,
        options.min_polyg_run,
    )?;
    let effective_params = TrimPolygTailsParams {
        schema_version: TRIM_POLYG_TAILS_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::from_has_r2(r2.is_some()),
        threads: tool.resources.threads,
        trim_polyg: options.trim_polyg,
        min_polyg_run: options.min_polyg_run,
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
            "trim_polyg": options.trim_polyg,
            "min_polyg_run": options.min_polyg_run,
        }),
        effective_params: serde_json::to_value(&effective_params)?,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "polyG tail trimming"),
    })
}

fn trim_polyg_command(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report: &Path,
    threads: u32,
    trim_polyg: bool,
    min_polyg_run: u32,
) -> Result<Vec<String>> {
    match tool_id {
        "fastp" => {
            let mut command = vec![
                "fastp".to_string(),
                "--json".to_string(),
                report.display().to_string(),
                "--thread".to_string(),
                threads.to_string(),
                "--in1".to_string(),
                r1.display().to_string(),
                "--out1".to_string(),
                output_r1.display().to_string(),
            ];
            if trim_polyg {
                command.push("--trim_poly_g".to_string());
                command.push("--poly_g_min_len".to_string());
                command.push(min_polyg_run.to_string());
            }
            if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
                command.push("--in2".to_string());
                command.push(r2.display().to_string());
                command.push("--out2".to_string());
                command.push(output_r2.display().to_string());
            }
            Ok(command)
        }
        "bbduk" => {
            let raw_stats = report.with_extension("stats.txt");
            let mut script = format!(
                "set -euo pipefail\nbbduk.sh in={} out={}",
                shell_quote_arg(&format!("in={}", r1.display())),
                shell_quote_arg(&format!("out={}", output_r1.display())),
            );
            if trim_polyg {
                script.push(' ');
                script.push_str(&shell_quote_arg(&format!("trimpolygright={min_polyg_run}")));
            }
            script.push(' ');
            script.push_str(&shell_quote_arg(&format!("stats={}", raw_stats.display())));
            if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
                script.push(' ');
                script.push_str(&shell_quote_arg(&format!("in2={}", r2.display())));
                script.push(' ');
                script.push_str(&shell_quote_arg(&format!("out2={}", output_r2.display())));
            }
            let report_payload = serde_json::json!({
                "schema_version": "bijux.fastq.trim_polyg_tails.report.v1",
                "stage_id": STAGE_ID.as_str(),
                "tool_id": tool_id,
                "trim_polyg": trim_polyg,
                "input_r1": r1,
                "input_r2": r2,
                "output_r1": output_r1,
                "output_r2": output_r2,
                "min_polyg_run": min_polyg_run,
                "raw_stats_path": raw_stats,
            });
            script.push_str(&format!(
                "\nprintf '%s\\n' {} > {}\n",
                shell_quote_str(&report_payload.to_string()),
                shell_quote_path(report),
            ));
            Ok(vec!["sh".to_string(), "-lc".to_string(), script])
        }
        _ => Err(anyhow!(
            "unsupported trim_polyg_tails tool for stage planning: {tool_id}"
        )),
    }
}

fn shell_quote_arg(value: &str) -> String {
    shell_quote_str(value)
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote_str(&path.display().to_string())
}

fn shell_quote_str(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
