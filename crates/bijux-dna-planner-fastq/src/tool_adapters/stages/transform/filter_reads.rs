#![allow(clippy::too_many_arguments)]

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{filter::FilterEffectiveParams, PairedMode};
use bijux_dna_domain_fastq::STAGE_FILTER_READS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_FILTER_READS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, Default)]
pub struct FilterPlanOptions {
    pub threads: Option<u32>,
    pub max_n: Option<u32>,
    pub max_n_fraction: Option<f64>,
    pub max_n_count: Option<u32>,
    pub low_complexity_threshold: Option<f64>,
    pub entropy_threshold: Option<f64>,
    pub kmer_ref: Option<PathBuf>,
    pub redundant_filters: Vec<String>,
    pub polyx_policy: Option<String>,
}

struct FilterPlanPaths {
    output_r1: PathBuf,
    output_r2: Option<PathBuf>,
    report_json: PathBuf,
    raw_backend_report: Option<PathBuf>,
    raw_backend_report_format: Option<&'static str>,
}

/// # Errors
/// Returns an error if any requested filter tool is not admitted for `fastq.filter_reads`.
pub fn normalize_filter_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
}

/// Build a filter plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_filter(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    options: &FilterPlanOptions,
) -> Result<StagePlanV1> {
    let output_name =
        filter_output_name(&tool.tool_id.0).ok_or_else(|| anyhow!("unsupported filter tool"))?;
    ensure_filter_option_support(&tool.tool_id.0, options)?;
    let effective_threads = options.threads.unwrap_or(tool.resources.threads).max(1);
    let paths = filter_plan_paths(&tool.tool_id.0, output_name, r2.is_some(), out_dir);
    let kmer_ref = options.kmer_ref.clone().map(|path| path.display().to_string());
    let effective_params =
        filter_effective_params(options, r2.is_some(), effective_threads, kmer_ref.as_ref());
    let inputs = filter_inputs(r1, r2);
    let outputs = filter_outputs(&paths);
    let command_template = filter_command_template(
        tool,
        r1,
        r2,
        &paths.output_r1,
        paths.output_r2.as_deref(),
        &paths.report_json,
        paths.raw_backend_report.as_deref(),
        effective_threads,
        options,
    )?;
    let mut resources = tool.resources.clone();
    resources.threads = effective_threads;
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
        command: bijux_dna_core::prelude::CommandSpecV1 { template: command_template },
        resources,
        io: StageIO { inputs, outputs },
        out_dir: out_dir.to_path_buf(),
        params: filter_plan_params(
            &tool.tool_id.0,
            r1,
            r2,
            &paths,
            options,
            effective_threads,
            kmer_ref.as_ref(),
        ),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize filter effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    })
}

fn filter_plan_paths(
    tool_id: &str,
    output_name: &str,
    paired: bool,
    out_dir: &Path,
) -> FilterPlanPaths {
    let output_r1 =
        if paired { out_dir.join(format!("R1.{output_name}")) } else { out_dir.join(output_name) };
    let (raw_backend_report, raw_backend_report_format) =
        raw_backend_report_contract(tool_id, out_dir);
    FilterPlanPaths {
        output_r1,
        output_r2: paired.then(|| out_dir.join(format!("R2.{output_name}"))),
        report_json: out_dir.join("filter_report.json"),
        raw_backend_report,
        raw_backend_report_format,
    }
}

fn filter_effective_params(
    options: &FilterPlanOptions,
    paired: bool,
    effective_threads: u32,
    kmer_ref: Option<&String>,
) -> FilterEffectiveParams {
    FilterEffectiveParams {
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: effective_threads,
        max_n: options.max_n,
        max_n_fraction: options.max_n_fraction,
        max_n_count: options.max_n_count.or(options.max_n),
        low_complexity_threshold: options.low_complexity_threshold,
        entropy_threshold: options.entropy_threshold,
        contaminant_db: kmer_ref.cloned(),
        n_policy: None,
        polyx_policy: options.polyx_policy.clone(),
        damage_mode: None,
    }
}

fn filter_inputs(r1: &Path, r2: Option<&Path>) -> Vec<ArtifactRef> {
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
    inputs
}

fn filter_outputs(paths: &FilterPlanPaths) -> Vec<ArtifactRef> {
    let mut outputs = vec![ArtifactRef::required(
        ArtifactId::from_static("filtered_reads_r1"),
        paths.output_r1.clone(),
        ArtifactRole::Reads,
    )];
    if let Some(output_r2) = &paths.output_r2 {
        outputs.push(ArtifactRef::required(
            ArtifactId::from_static("filtered_reads_r2"),
            output_r2.clone(),
            ArtifactRole::Reads,
        ));
    }
    outputs.push(ArtifactRef::required(
        ArtifactId::from_static("report_json"),
        paths.report_json.clone(),
        ArtifactRole::ReportJson,
    ));
    outputs
}

fn filter_plan_params(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    paths: &FilterPlanPaths,
    options: &FilterPlanOptions,
    effective_threads: u32,
    kmer_ref: Option<&String>,
) -> serde_json::Value {
    serde_json::json!({
        "tool": tool_id,
        "threads": effective_threads,
        "input_r1": r1,
        "input_r2": r2,
        "output_r1": paths.output_r1,
        "output_r2": paths.output_r2,
        "report_json": paths.report_json,
        "raw_backend_report": paths.raw_backend_report,
        "raw_backend_report_format": paths.raw_backend_report_format,
        "max_n": options.max_n,
        "max_n_fraction": options.max_n_fraction,
        "max_n_count": options.max_n_count,
        "low_complexity_threshold": options.low_complexity_threshold,
        "entropy_threshold": options.entropy_threshold,
        "kmer_ref": kmer_ref,
        "redundant_filters": options.redundant_filters,
        "polyx_policy": options.polyx_policy,
    })
}

fn filter_command_template(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    raw_backend_report: Option<&Path>,
    effective_threads: u32,
    options: &FilterPlanOptions,
) -> Result<Vec<String>> {
    if tool.tool_id.as_str() == "fastp" {
        let mut command = vec![
            "fastp".to_string(),
            "--in1".to_string(),
            r1.display().to_string(),
            "--out1".to_string(),
            output_r1.display().to_string(),
            "--thread".to_string(),
            effective_threads.to_string(),
        ];
        if let Some(raw_backend_report) = raw_backend_report {
            command.extend(["--json".to_string(), raw_backend_report.display().to_string()]);
        }
        if let Some(limit) = options.max_n_count.or(options.max_n) {
            command.extend(["--n_base_limit".to_string(), limit.to_string()]);
        }
        if let Some(threshold) = options.low_complexity_threshold.or(options.entropy_threshold) {
            command.push("--low_complexity_filter".to_string());
            command.extend(["--complexity_threshold".to_string(), threshold.to_string()]);
        }
        if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
            command.extend([
                "--in2".to_string(),
                r2.display().to_string(),
                "--out2".to_string(),
                output_r2.display().to_string(),
            ]);
        }
        return Ok(command);
    }
    crate::tool_adapters::template_render::render_command_template(
        &tool.command.template,
        &[
            ("reads", Some(r1.display().to_string())),
            ("reads_r1", Some(r1.display().to_string())),
            ("reads_r2", r2.map(|path| path.display().to_string())),
            ("filtered_reads", Some(output_r1.display().to_string())),
            ("filtered_reads_r1", Some(output_r1.display().to_string())),
            ("filtered_reads_r2", output_r2.map(|path| path.display().to_string())),
            ("report_json", Some(report_json.display().to_string())),
            ("filter_report_json", Some(report_json.display().to_string())),
            ("raw_backend_report", raw_backend_report.map(|path| path.display().to_string())),
            ("trimmed_reads", Some(output_r1.display().to_string())),
            ("trimmed_reads_r1", Some(output_r1.display().to_string())),
            ("trimmed_reads_r2", output_r2.map(|path| path.display().to_string())),
        ],
    )
}

fn filter_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fastp" => Some("fastp.fastq.gz"),
        "prinseq" => Some("prinseq_good.fastq"),
        "seqkit" => Some("seqkit.fastq.gz"),
        "bbduk" => Some("bbduk.fastq.gz"),
        _ => None,
    }
}

fn raw_backend_report_contract(
    tool: &str,
    out_dir: &Path,
) -> (Option<PathBuf>, Option<&'static str>) {
    match tool {
        "fastp" => (Some(out_dir.join("fastp.filter.json")), Some("fastp_json")),
        "bbduk" => (Some(out_dir.join("bbduk.filter.stats")), Some("bbduk_stats")),
        _ => (None, None),
    }
}

fn ensure_filter_option_support(tool_id: &str, options: &FilterPlanOptions) -> Result<()> {
    if tool_id == "fastp" {
        if options.kmer_ref.is_some() {
            return Err(anyhow!(
                "fastp filter planning does not support contaminant k-mer reference filtering"
            ));
        }
        if options.max_n_fraction.is_some() {
            return Err(anyhow!(
                "fastp filter planning does not support max_n_fraction without a count translation"
            ));
        }
    }
    Ok(())
}

#[must_use]
pub fn default_kmer_ref() -> Option<PathBuf> {
    let dir = bijux_dna_domain_fastq::contaminant_references_dir();
    let entries = std::fs::read_dir(dir).ok()?;
    let mut fasta = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("fasta") {
            fasta.push(path);
        }
    }
    fasta.sort();
    fasta.into_iter().next()
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
