#![allow(clippy::too_many_arguments)]

use std::fmt::Write as _;
use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::defaults::trim_polyg_tails_defaults;
use bijux_dna_domain_fastq::params::trim::{TrimPolygTailsParams, TRIM_POLYG_TAILS_SCHEMA_VERSION};
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_POLYG_TAILS;
use bijux_dna_domain_fastq::{TrimPolygReportV1, TRIM_POLYG_REPORT_SCHEMA_VERSION};
use bijux_dna_stage_contract::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1,
};

pub const STAGE_ID: StageId = STAGE_TRIM_POLYG_TAILS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimPolygPlanOptions {
    pub threads: Option<u32>,
    pub trim_polyg: bool,
    pub min_polyg_run: u32,
}

impl Default for TrimPolygPlanOptions {
    fn default() -> Self {
        Self { threads: None, trim_polyg: true, min_polyg_run: 10 }
    }
}

fn output_name(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "fastp" => Some("polyg.fastp.fastq.gz"),
        "bbduk" => Some("polyg.bbduk.fastq.gz"),
        _ => None,
    }
}

fn raw_backend_report_artifact(
    report: &Path,
    tool_id: &str,
) -> Result<(std::path::PathBuf, &'static str)> {
    match tool_id {
        "fastp" => Ok((report.with_extension("fastp.json"), "fastp_json")),
        "bbduk" => Ok((report.with_extension("stats.txt"), "bbduk_stats")),
        _ => Err(anyhow!(
            "unsupported trim_polyg_tails raw report artifact for stage planning: {tool_id}"
        )),
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
    let output_r1 =
        if r2.is_some() { out_dir.join(format!("R1.{out_name}")) } else { out_dir.join(out_name) };
    let output_r2 = r2.map(|_| out_dir.join(format!("R2.{out_name}")));
    let report = out_dir.join("trim_polyg_tails_report.json");
    let (raw_backend_report, raw_backend_report_format) =
        raw_backend_report_artifact(&report, tool.tool_id.as_str())?;
    let default_threads = trim_polyg_tails_defaults(r2.is_some()).threads;
    let effective_threads = options.threads.unwrap_or(default_threads).max(1);
    let command_template = trim_polyg_command(
        &tool.tool_id.0,
        r1,
        r2,
        &output_r1,
        output_r2.as_deref(),
        &report,
        &raw_backend_report,
        raw_backend_report_format,
        effective_threads,
        options.trim_polyg,
        options.min_polyg_run,
    )?;
    let effective_params = TrimPolygTailsParams {
        schema_version: TRIM_POLYG_TAILS_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::from_has_r2(r2.is_some()),
        threads: effective_threads,
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
    if let Some(raw_backend_output) =
        trim_polyg_raw_backend_output(tool.tool_id.as_str(), &raw_backend_report)
    {
        outputs.push(raw_backend_output);
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
        resources: {
            let mut resources = tool.resources.clone();
            resources.threads = effective_threads;
            resources
        },
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input_r1": r1,
            "input_r2": r2,
            "output_r1": output_r1,
            "output_r2": output_r2,
            "report_json": report,
            "threads": effective_threads,
            "trim_polyg": options.trim_polyg,
            "min_polyg_run": options.min_polyg_run,
            "raw_backend_report": raw_backend_report,
            "raw_backend_report_format": raw_backend_report_format,
        }),
        effective_params: serde_json::to_value(&effective_params)?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
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
    raw_backend_report: &Path,
    raw_backend_report_format: &str,
    threads: u32,
    trim_polyg: bool,
    min_polyg_run: u32,
) -> Result<Vec<String>> {
    match tool_id {
        "fastp" => {
            let mut command = vec![
                "fastp".to_string(),
                "--json".to_string(),
                raw_backend_report.display().to_string(),
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
            Ok(wrap_polyg_command_with_report(
                tool_id,
                &command,
                r1,
                r2,
                output_r1,
                output_r2,
                report,
                raw_backend_report,
                raw_backend_report_format,
                threads,
                trim_polyg,
                min_polyg_run,
            ))
        }
        "bbduk" => {
            let mut command = vec![
                "bbduk".to_string(),
                format!("in={}", r1.display()),
                format!("out={}", output_r1.display()),
                format!("threads={threads}"),
            ];
            if trim_polyg {
                command.push(format!("trimpolygright={min_polyg_run}"));
            }
            command.push(format!("stats={}", raw_backend_report.display()));
            if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
                command.push(format!("in2={}", r2.display()));
                command.push(format!("out2={}", output_r2.display()));
            }
            Ok(wrap_polyg_command_with_report(
                tool_id,
                &command,
                r1,
                r2,
                output_r1,
                output_r2,
                report,
                raw_backend_report,
                raw_backend_report_format,
                threads,
                trim_polyg,
                min_polyg_run,
            ))
        }
        _ => Err(anyhow!("unsupported trim_polyg_tails tool for stage planning: {tool_id}")),
    }
}

fn trim_polyg_raw_backend_output(tool_id: &str, raw_backend_report: &Path) -> Option<ArtifactRef> {
    match tool_id {
        "fastp" => Some(ArtifactRef::optional(
            ArtifactId::from_static("raw_backend_report_json"),
            raw_backend_report.to_path_buf(),
            ArtifactRole::ReportJson,
        )),
        "bbduk" => Some(ArtifactRef::optional(
            ArtifactId::from_static("raw_backend_report_txt"),
            raw_backend_report.to_path_buf(),
            ArtifactRole::Log,
        )),
        _ => None,
    }
}

fn wrap_polyg_command_with_report(
    tool_id: &str,
    command: &[String],
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report: &Path,
    raw_report: &Path,
    raw_report_format: &str,
    threads: u32,
    trim_polyg: bool,
    min_polyg_run: u32,
) -> Vec<String> {
    let payload = TrimPolygReportV1 {
        schema_version: TRIM_POLYG_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_ID.as_str().to_string(),
        stage_id: STAGE_ID.as_str().to_string(),
        tool_id: tool_id.to_string(),
        paired_mode: PairedMode::from_has_r2(r2.is_some()),
        threads,
        trim_polyg,
        min_polyg_run,
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        reads_in: None,
        reads_out: None,
        bases_in: None,
        bases_out: None,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: None,
        mean_q_after: None,
        trimmed_tail_count: None,
        bases_trimmed_polyg: None,
        polyx_bank_id: None,
        polyx_bank_hash: None,
        polyx_preset: None,
        runtime_s: None,
        memory_mb: None,
        raw_backend_report: Some(raw_report.display().to_string()),
        raw_backend_report_format: Some(raw_report_format.to_string()),
        backend_metrics: None,
    };
    let mut script = format!("set -eu\n{}\n", shell_join(command));
    let payload_json = serde_json::to_string(&payload)
        .unwrap_or_else(|_| unreachable!("serializing trim polyg report cannot fail"));
    script
        .write_fmt(format_args!(
            "printf '%s\\n' {} > {}\n",
            shell_quote_str(&payload_json),
            shell_quote_path(report),
        ))
        .unwrap_or_else(|_| unreachable!("writing to String cannot fail"));
    vec!["sh".to_string(), "-lc".to_string(), script]
}

fn shell_join(command: &[String]) -> String {
    command.iter().map(|part| shell_quote_arg(part)).collect::<Vec<_>>().join(" ")
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

#[cfg(test)]
mod tests {
    use super::{
        plan_trim_polyg_tails_with_options, raw_backend_report_artifact, TrimPolygPlanOptions,
    };
    use bijux_dna_core::prelude::{
        CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
    };
    use std::path::Path;

    fn tool(tool_id: &'static str) -> ToolExecutionSpecV1 {
        ToolExecutionSpecV1 {
            tool_id: ToolId::from_static(tool_id),
            tool_version: "1.0.0".into(),
            image: ContainerImageRefV1 { image: format!("{tool_id}:latest"), digest: None },
            command: CommandSpecV1 { template: vec![tool_id.to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 4,
            },
        }
    }

    #[test]
    fn raw_backend_report_artifact_uses_backend_specific_names() {
        let report = Path::new("out/trim_polyg_tails_report.json");
        let (fastp_path, fastp_format) =
            raw_backend_report_artifact(report, "fastp").expect("fastp artifact");
        assert_eq!(fastp_path, Path::new("out/trim_polyg_tails_report.fastp.json"));
        assert_eq!(fastp_format, "fastp_json");

        let (bbduk_path, bbduk_format) =
            raw_backend_report_artifact(report, "bbduk").expect("bbduk artifact");
        assert_eq!(bbduk_path, Path::new("out/trim_polyg_tails_report.stats.txt"));
        assert_eq!(bbduk_format, "bbduk_stats");
    }

    #[test]
    fn plan_trim_polyg_tails_records_governed_report_shape() {
        let plan = plan_trim_polyg_tails_with_options(
            &tool("fastp"),
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out"),
            &TrimPolygPlanOptions { threads: None, trim_polyg: true, min_polyg_run: 12 },
        )
        .expect("plan");

        let params = plan.params.as_object().expect("params object");
        let script = &plan.command.template[2];
        assert_eq!(
            params.get("raw_backend_report_format").and_then(serde_json::Value::as_str),
            Some("fastp_json")
        );
        assert!(script.contains("trim_polyg_tails_report.fastp.json"));
        assert!(script.contains("\"schema_version\":\"bijux.fastq.trim_polyg_tails.report.v2\""));
        assert!(script.contains("\"paired_mode\":\"paired_end\""));
        assert!(script.contains("\"raw_backend_report_format\":\"fastp_json\""));
    }

    #[test]
    fn plan_trim_polyg_tails_honors_thread_override() {
        let plan = plan_trim_polyg_tails_with_options(
            &tool("fastp"),
            Path::new("reads.fastq.gz"),
            None,
            Path::new("out"),
            &TrimPolygPlanOptions { threads: Some(7), trim_polyg: true, min_polyg_run: 10 },
        )
        .expect("plan");

        assert_eq!(plan.resources.threads, 7);
        assert_eq!(plan.effective_params["threads"], serde_json::json!(7));
        assert_eq!(plan.params["threads"], serde_json::json!(7));
        assert!(plan.command.template[2].contains("'--thread' '7'"));
    }
}
