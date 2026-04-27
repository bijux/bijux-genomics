#![allow(
    clippy::format_push_string,
    clippy::too_many_arguments,
    clippy::uninlined_format_args,
    clippy::unnecessary_wraps
)]

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, CommandSpecV1, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{trim::TrimEffectiveParams, PairedMode};
use bijux_dna_domain_fastq::STAGE_TRIM_READS;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

mod config;
mod reporting;

use config::{ensure_trim_option_support, normalize_trim_threads};
pub use config::{
    resolve_config, trim_output_name, validate_trim_toolset_support, TrimEffectiveConfig,
    TrimPlanOptions, TrimUserConfig,
};
use reporting::{
    move_first_existing_output_script, raw_backend_report_path, trim_raw_backend_output,
    wrap_trim_command_with_report, wrap_trim_shell_script_with_report, write_trim_report_script,
};

pub const STAGE_ID: StageId = STAGE_TRIM_READS;
pub const STAGE_VERSION: StageVersion = StageVersion(1);
const FALLBACK_TRIM_ADAPTER_R1: &str = "CGTACGATTCGAGCTAGTCCGATGCTTACGATCGTTCAGAGTAC";
const FALLBACK_TRIM_ADAPTER_R2: &str = "TGCATCGACTAGCGTTACGTCAGTATCGGATCAGTTCGATGACA";

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
    plan_with_options(
        tool,
        r1,
        r2,
        out_dir,
        adapter_bank,
        polyx_bank,
        contaminant_bank,
        &TrimPlanOptions::default(),
    )
}

pub fn plan_with_options(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: Option<&Path>,
    out_dir: &Path,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<StagePlanV1> {
    let output_name =
        trim_output_name(&tool.tool_id.0).ok_or_else(|| anyhow!("unsupported trim tool"))?;
    if tool.tool_id.as_str() == "seqpurge" && r2.is_none() {
        return Err(anyhow!("seqpurge trim planning requires paired-end reads"));
    }
    ensure_trim_option_support(&tool.tool_id.0, options)?;
    let effective_threads = normalize_trim_threads(
        tool.tool_id.as_str(),
        options.resolved_threads(tool.resources.threads),
    );
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
        "output_r2": output_r2,
        "threads": effective_threads,
        "min_length": options.resolved_min_length(),
        "quality_cutoff": options.quality_cutoff,
        "n_policy": options.resolved_n_policy(),
        "adapter_policy": options.resolved_adapter_policy(),
        "polyx_policy": options.resolved_polyx_policy(),
        "contaminant_policy": options.resolved_contaminant_policy(),
    });
    if options.resolved_adapter_policy() != "none" {
        if let Some(adapter_bank) = adapter_bank {
            if let Some(map) = params.as_object_mut() {
                map.insert("adapter_bank".to_string(), adapter_bank.clone());
            }
        }
    }
    if options.resolved_polyx_policy() != "none" {
        if let Some(polyx_bank) = polyx_bank {
            if let Some(map) = params.as_object_mut() {
                map.insert("polyx_bank".to_string(), polyx_bank.clone());
            }
        }
    }
    if options.resolved_contaminant_policy() != "none" {
        if let Some(contaminant_bank) = contaminant_bank {
            if let Some(map) = params.as_object_mut() {
                map.insert("contaminant_bank".to_string(), contaminant_bank.clone());
            }
        }
    }
    let effective_params = TrimEffectiveParams {
        paired_mode: if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: effective_threads,
        min_len: options.resolved_min_length(),
        q_cutoff: options.quality_cutoff,
        adapter_policy: options.resolved_adapter_policy(),
        damage_mode: None,
        polyx_policy: Some(options.resolved_polyx_policy()),
        n_policy: Some(options.resolved_n_policy()),
        contaminant_policy: Some(options.resolved_contaminant_policy()),
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
    let report_json = out_dir.join("trim_report.json");
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
        report_json.clone(),
        ArtifactRole::ReportJson,
    ));
    if let Some(raw_backend_output) = trim_raw_backend_output(tool.tool_id.as_str(), &report_json) {
        outputs.push(raw_backend_output);
    }
    let command_template = trim_command_template(
        tool,
        r1,
        r2,
        &output_r1,
        output_r2.as_deref(),
        &report_json,
        adapter_bank,
        polyx_bank,
        contaminant_bank,
        options,
    )?;
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
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let adapter_policy = options.resolved_adapter_policy();
    let adapter_sequences = enabled_adapter_sequences(adapter_bank);
    let polyx_policy = options.resolved_polyx_policy();
    let effective_threads = normalize_trim_threads(
        tool.tool_id.as_str(),
        options.resolved_threads(tool.resources.threads),
    );
    if tool.tool_id.as_str() == "fastp" {
        let raw_backend_report = raw_backend_report_path(report_json, "fastp", "json");
        let mut command = vec![
            "fastp".to_string(),
            "--in1".to_string(),
            r1.display().to_string(),
            "--out1".to_string(),
            output_r1.display().to_string(),
            "--json".to_string(),
            raw_backend_report.display().to_string(),
            "--thread".to_string(),
            effective_threads.to_string(),
        ];
        if let Some(min_length) = options.min_length {
            command.extend(["--length_required".to_string(), min_length.to_string()]);
        }
        if let Some(quality_cutoff) = options.quality_cutoff {
            command.extend(["--qualified_quality_phred".to_string(), quality_cutoff.to_string()]);
        }
        if options.resolved_n_policy() == "drop" {
            command.extend(["--n_base_limit".to_string(), "0".to_string()]);
        }
        if let Some(adapter_sequence) = adapter_sequences.first() {
            if adapter_policy != "none" && adapter_policy != "auto" {
                command.extend(["--adapter_sequence".to_string(), adapter_sequence.clone()]);
                if r2.is_some() {
                    command.extend([
                        "--adapter_sequence_r2".to_string(),
                        adapter_sequences
                            .get(1)
                            .cloned()
                            .unwrap_or_else(|| adapter_sequence.clone()),
                    ]);
                }
            }
        }
        if polyx_policy == "trim" || (polyx_policy == "bank" && polyx_bank.is_some()) {
            command.push("--trim_poly_x".to_string());
        }
        if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
            command.extend([
                "--in2".to_string(),
                r2.display().to_string(),
                "--out2".to_string(),
                output_r2.display().to_string(),
            ]);
            if adapter_policy == "auto" && adapter_sequences.is_empty() {
                command.push("--detect_adapter_for_pe".to_string());
            }
        }
        return Ok(wrap_trim_command_with_report(
            "fastp",
            &command,
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            adapter_bank,
            polyx_bank,
            contaminant_bank,
            options,
            Some(raw_backend_report.as_path()),
            Some("fastp_json"),
        ));
    }
    if tool.tool_id.as_str() == "cutadapt" {
        return cutadapt_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            adapter_bank,
            options,
        );
    }
    if tool.tool_id.as_str() == "atropos" {
        return atropos_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            adapter_bank,
            options,
        );
    }
    if tool.tool_id.as_str() == "bbduk" {
        return bbduk_trim_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            contaminant_bank,
            options,
        );
    }
    if tool.tool_id.as_str() == "adapterremoval" {
        return adapterremoval_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            adapter_bank,
            options,
        );
    }
    if tool.tool_id.as_str() == "alientrimmer" {
        return alientrimmer_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            adapter_bank,
            options,
        );
    }
    if tool.tool_id.as_str() == "fastx_clipper" {
        return fastx_clipper_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            adapter_bank,
            options,
        );
    }
    if tool.tool_id.as_str() == "skewer" {
        return skewer_trim_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            adapter_bank,
            options,
        );
    }
    if tool.tool_id.as_str() == "leehom" {
        return leehom_trim_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            adapter_bank,
            options,
        );
    }
    if tool.tool_id.as_str() == "trimmomatic" {
        return trimmomatic_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            options,
        );
    }
    if tool.tool_id.as_str() == "trim_galore" {
        return trim_galore_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            adapter_bank,
            options,
        );
    }
    if tool.tool_id.as_str() == "seqkit" {
        return seqkit_trim_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            options,
        );
    }
    if tool.tool_id.as_str() == "seqpurge" {
        return seqpurge_trim_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            options,
        );
    }
    if tool.tool_id.as_str() == "prinseq" {
        return prinseq_trim_command_template(
            r1,
            r2,
            output_r1,
            output_r2,
            report_json,
            effective_threads,
            options,
        );
    }
    let rendered = crate::tool_adapters::template_render::render_command_template(
        &tool.command.template,
        &[
            ("reads", Some(r1.display().to_string())),
            ("reads_r1", Some(r1.display().to_string())),
            ("reads_r2", r2.map(|path| path.display().to_string())),
            ("trimmed_reads", Some(output_r1.display().to_string())),
            ("filtered_reads", Some(output_r1.display().to_string())),
            ("trimmed_reads_dir", output_r1.parent().map(|path| path.display().to_string())),
            ("trimmed_reads_r1", Some(output_r1.display().to_string())),
            ("filtered_reads_r1", Some(output_r1.display().to_string())),
            ("trimmed_reads_r2", output_r2.map(|path| path.display().to_string())),
            ("filtered_reads_r2", output_r2.map(|path| path.display().to_string())),
            ("report_json", Some(report_json.display().to_string())),
            ("threads", Some(effective_threads.to_string())),
        ],
    )?;
    Ok(wrap_trim_command_with_report(
        &tool.tool_id.0,
        &rendered,
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        effective_threads,
        adapter_bank,
        polyx_bank,
        contaminant_bank,
        options,
        None,
        None,
    ))
}

fn seqkit_trim_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let min_length = options.min_length.unwrap_or(1);
    let mut script = format!(
        "set -eu\nseqkit seq -m {min_length} -o {} {}\n",
        shell_quote_path(output_r1),
        shell_quote_path(r1),
    );
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        script.push_str(&format!(
            "seqkit seq -m {min_length} -o {} {}\n",
            shell_quote_path(output_r2),
            shell_quote_path(r2),
        ));
    }
    script.push_str(&write_trim_report_script(
        "seqkit",
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        None,
        None,
        None,
        options,
        None,
        None,
    ));
    Ok(vec!["sh".to_string(), "-lc".to_string(), script])
}

fn seqpurge_trim_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let mut command = vec![
        "seqpurge".to_string(),
        "-threads".to_string(),
        threads.max(1).to_string(),
        "-in1".to_string(),
        r1.display().to_string(),
        "-out1".to_string(),
        output_r1.display().to_string(),
    ];
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        command.extend([
            "-in2".to_string(),
            r2.display().to_string(),
            "-out2".to_string(),
            output_r2.display().to_string(),
        ]);
    }
    if let Some(min_length) = options.min_length {
        command.extend(["-min_len".to_string(), min_length.to_string()]);
    }
    Ok(wrap_trim_command_with_report(
        "seqpurge",
        &command,
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        None,
        None,
        None,
        options,
        None,
        None,
    ))
}

fn prinseq_trim_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let mut command = vec![
        "prinseq++".to_string(),
        "-threads".to_string(),
        threads.max(1).to_string(),
        "-fastq".to_string(),
        r1.display().to_string(),
        "-out_good".to_string(),
        output_r1.display().to_string(),
        "-out_bad".to_string(),
        "/dev/null".to_string(),
    ];
    if let Some(min_length) = options.min_length {
        command.extend(["-min_len".to_string(), min_length.to_string()]);
    }
    if let Some(quality_cutoff) = options.quality_cutoff {
        command.extend([
            "-trim_qual_left".to_string(),
            quality_cutoff.to_string(),
            "-trim_qual_right".to_string(),
            quality_cutoff.to_string(),
        ]);
    }
    if options.resolved_n_policy() == "drop" {
        command.extend(["-ns_max_n".to_string(), "0".to_string()]);
    }
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        command.extend([
            "-fastq2".to_string(),
            r2.display().to_string(),
            "-out_good2".to_string(),
            output_r2.display().to_string(),
            "-out_bad2".to_string(),
            "/dev/null".to_string(),
            "-out_single".to_string(),
            "/dev/null".to_string(),
            "-out_single2".to_string(),
            "/dev/null".to_string(),
        ]);
    }
    Ok(wrap_trim_command_with_report(
        "prinseq",
        &command,
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        None,
        None,
        None,
        options,
        None,
        None,
    ))
}

fn cutadapt_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let mut command =
        vec!["cutadapt".to_string(), "--cores".to_string(), threads.max(1).to_string()];
    let raw_backend_report = raw_backend_report_path(report_json, "cutadapt", "json");
    if matches!(options.resolved_adapter_policy().as_str(), "bank" | "ancient_strict") {
        for adapter in enabled_adapter_sequences(adapter_bank) {
            command.extend(["-a".to_string(), adapter.clone()]);
            if r2.is_some() {
                command.extend(["-A".to_string(), adapter]);
            }
        }
    }
    if let Some(min_length) = options.min_length {
        command.extend(["-m".to_string(), min_length.to_string()]);
    }
    if let Some(quality_cutoff) = options.quality_cutoff {
        command.extend(["-q".to_string(), quality_cutoff.to_string()]);
    }
    if options.resolved_n_policy() == "drop" {
        command.extend(["--max-n".to_string(), "0".to_string()]);
    }
    command.extend(["--json".to_string(), raw_backend_report.display().to_string()]);
    command.extend(["-o".to_string(), output_r1.display().to_string()]);
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        command.extend([
            "-p".to_string(),
            output_r2.display().to_string(),
            r1.display().to_string(),
            r2.display().to_string(),
        ]);
    } else {
        command.push(r1.display().to_string());
    }
    Ok(wrap_trim_command_with_report(
        "cutadapt",
        &command,
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        adapter_bank,
        None,
        None,
        options,
        Some(raw_backend_report.as_path()),
        Some("cutadapt_json"),
    ))
}

fn bbduk_trim_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    contaminant_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let contaminant_ref = report_json
        .parent()
        .ok_or_else(|| anyhow!("trim report path must have a parent directory"))?
        .join("bbduk_contaminants.fa");
    let raw_backend_report = raw_backend_report_path(report_json, "bbduk", "stats.txt");
    let mut command = vec![
        "bbduk".to_string(),
        format!("in={}", r1.display()),
        format!("out={}", output_r1.display()),
        format!("stats={}", raw_backend_report.display()),
        format!("threads={}", threads.max(1)),
    ];
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        command.push(format!("in2={}", r2.display()));
        command.push(format!("out2={}", output_r2.display()));
    }
    if let Some(min_length) = options.min_length {
        command.push(format!("minlen={min_length}"));
    }
    if let Some(quality_cutoff) = options.quality_cutoff {
        command.push("qtrim=rl".to_string());
        command.push(format!("trimq={quality_cutoff}"));
    }
    if options.resolved_n_policy() == "drop" {
        command.push("maxns=0".to_string());
    }
    if options.resolved_contaminant_policy() == "bank" {
        command.push(format!("ref={}", contaminant_ref.display()));
        command.push("k=31".to_string());
        command.push("hdist=1".to_string());
    }
    let wrapped = wrap_trim_command_with_report(
        "bbduk",
        &command,
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        None,
        None,
        contaminant_bank,
        options,
        Some(raw_backend_report.as_path()),
        Some("bbduk_stats"),
    );
    if options.resolved_contaminant_policy() != "bank" {
        return Ok(wrapped);
    }
    let contaminant_fasta = contaminant_bank_fasta(contaminant_bank)?;
    let script = format!(
        "set -eu\ncat <<'EOF' > {}\n{}\nEOF\n{}\n",
        shell_quote_path(&contaminant_ref),
        contaminant_fasta.trim_end(),
        shell_join(&wrapped),
    );
    Ok(vec!["sh".to_string(), "-lc".to_string(), script])
}

fn contaminant_bank_fasta(contaminant_bank: Option<&serde_json::Value>) -> Result<String> {
    let contaminant_bank = contaminant_bank
        .ok_or_else(|| anyhow!("trim contaminant_policy=bank requires a contaminant bank"))?;
    let mut entries = Vec::new();
    if let Some(enabled_entries) =
        contaminant_bank.get("enabled_entries").and_then(serde_json::Value::as_array)
    {
        for entry in enabled_entries {
            let Some(id) = entry.get("id").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let Some(sequence) = entry.get("sequence").and_then(serde_json::Value::as_str) else {
                continue;
            };
            entries.push(format!(">{id}\n{sequence}"));
        }
    }
    if let Some(references) =
        contaminant_bank.get("references").and_then(serde_json::Value::as_array)
    {
        for reference in references {
            let Some(id) = reference.get("id").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let Some(fasta) = reference.get("fasta").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let fasta = fasta.trim();
            if fasta.is_empty() {
                continue;
            }
            if fasta.starts_with('>') {
                entries.push(fasta.to_string());
            } else {
                entries.push(format!(">{id}\n{fasta}"));
            }
        }
    }
    if entries.is_empty() {
        return Err(anyhow!(
            "trim contaminant_policy=bank requires at least one contaminant sequence or reference"
        ));
    }
    Ok(entries.join("\n"))
}

fn atropos_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let (adapter_r1, adapter_r2) = resolved_adapter_pair(adapter_bank);
    let mut command =
        vec!["atropos".to_string(), "trim".to_string(), "-T".to_string(), threads.to_string()];
    if matches!(options.resolved_adapter_policy().as_str(), "bank" | "ancient_strict") {
        for adapter in enabled_adapter_sequences(adapter_bank) {
            command.extend(["-a".to_string(), adapter.clone()]);
            if r2.is_some() {
                command.extend(["-A".to_string(), adapter]);
            }
        }
    } else {
        command.extend(["-a".to_string(), adapter_r1.clone()]);
        if r2.is_some() {
            command.extend(["-A".to_string(), adapter_r2]);
        }
        command.extend(["-O".to_string(), adapter_r1.len().to_string()]);
    }
    if let Some(quality_cutoff) = options.quality_cutoff {
        command.extend(["-q".to_string(), quality_cutoff.to_string()]);
    }
    if let Some(min_length) = options.min_length {
        command.extend(["-m".to_string(), min_length.to_string()]);
    }
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        command.extend([
            "-pe1".to_string(),
            r1.display().to_string(),
            "-pe2".to_string(),
            r2.display().to_string(),
            "-o".to_string(),
            output_r1.display().to_string(),
            "-p".to_string(),
            output_r2.display().to_string(),
        ]);
    } else {
        command.extend([
            "-se".to_string(),
            r1.display().to_string(),
            "-o".to_string(),
            output_r1.display().to_string(),
        ]);
    }
    Ok(wrap_trim_command_with_report(
        "atropos",
        &command,
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        adapter_bank,
        None,
        None,
        options,
        None,
        None,
    ))
}

fn fastx_clipper_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let adapter_sequence = resolved_adapter_pair(adapter_bank).0;
    let mut script = String::from("set -eu\n");
    script.push_str(&fastx_clipper_single_command(
        r1,
        output_r1,
        &adapter_sequence,
        options.min_length,
    ));
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        script.push_str(&fastx_clipper_single_command(
            r2,
            output_r2,
            &adapter_sequence,
            options.min_length,
        ));
    }
    script.push_str(&write_trim_report_script(
        "fastx_clipper",
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        adapter_bank,
        None,
        None,
        options,
        None,
        None,
    ));
    Ok(wrap_trim_shell_script_with_report(&script, output_r1, output_r2, report_json, None))
}

fn fastx_clipper_single_command(
    input: &Path,
    output: &Path,
    adapter_sequence: &str,
    min_length: Option<u32>,
) -> String {
    let mut command = if input.extension().is_some_and(|ext| ext == "gz") {
        format!(
            "gzip -dc {} | fastx_clipper -Q33 -a {} -o {}",
            shell_quote_path(input),
            shell_quote_str(adapter_sequence),
            shell_quote_path(output),
        )
    } else {
        format!(
            "fastx_clipper -Q33 -a {} -i {} -o {}",
            shell_quote_str(adapter_sequence),
            shell_quote_path(input),
            shell_quote_path(output),
        )
    };
    command.push_str(&format!(" -M {}", adapter_sequence.len()));
    if let Some(min_length) = min_length {
        command.push_str(&format!(" -l {min_length}"));
    }
    if output.extension().is_some_and(|ext| ext == "gz") {
        command.push_str(" -z");
    }
    command.push('\n');
    command
}

fn skewer_trim_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let output_dir = output_r1
        .parent()
        .ok_or_else(|| anyhow!("skewer output path must have a parent directory"))?;
    let prefix = output_dir.join("skewer");
    let (adapter_r1, adapter_r2) = resolved_adapter_pair(adapter_bank);

    let mut script = format!("set -eu\nmkdir -p {}\nskewer", shell_quote_path(output_dir));
    if r2.is_some() {
        script.push_str(" -m pe");
    } else {
        script.push_str(" -m tail");
    }
    script.push_str(&format!(
        " -t {} -o {} -x {}",
        threads.max(1),
        shell_quote_path(&prefix),
        shell_quote_str(&adapter_r1),
    ));
    if r2.is_some() {
        script.push_str(&format!(" -y {}", shell_quote_str(&adapter_r2)));
    }
    if let Some(min_length) = options.min_length {
        script.push_str(&format!(" -l {min_length}"));
    }
    if let Some(quality_cutoff) = options.quality_cutoff {
        script.push_str(&format!(" -q {quality_cutoff}"));
    }
    if output_r1.extension().is_some_and(|ext| ext == "gz") {
        script.push_str(" -z");
    }
    script.push(' ');
    script.push_str(&shell_quote_path(r1));
    if let Some(r2) = r2 {
        script.push(' ');
        script.push_str(&shell_quote_path(r2));
    }
    script.push('\n');
    if let Some(output_r2) = output_r2 {
        script.push_str(&move_first_existing_output_script(
            &[
                prefix.with_file_name("skewer-trimmed-pair1.fastq.gz"),
                prefix.with_file_name("skewer-trimmed-pair1.fastq"),
            ],
            output_r1,
            "skewer R1 output",
        ));
        script.push_str(&move_first_existing_output_script(
            &[
                prefix.with_file_name("skewer-trimmed-pair2.fastq.gz"),
                prefix.with_file_name("skewer-trimmed-pair2.fastq"),
            ],
            output_r2,
            "skewer R2 output",
        ));
    } else {
        script.push_str(&move_first_existing_output_script(
            &[
                prefix.with_file_name("skewer-trimmed.fastq.gz"),
                prefix.with_file_name("skewer-trimmed.fastq"),
            ],
            output_r1,
            "skewer output",
        ));
    }
    script.push_str(&write_trim_report_script(
        "skewer",
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        adapter_bank,
        None,
        None,
        options,
        None,
        None,
    ));
    Ok(vec!["sh".to_string(), "-lc".to_string(), script])
}

fn leehom_trim_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let output_dir = output_r1
        .parent()
        .ok_or_else(|| anyhow!("leehom output path must have a parent directory"))?;
    let log_path = raw_backend_report_path(report_json, "leehom", "log");
    let (adapter_r1, adapter_r2) = resolved_adapter_pair(adapter_bank);
    let mut script = format!(
        "set -eu\nmkdir -p {}\ncd {}\nleehom",
        shell_quote_path(output_dir),
        shell_quote_path(output_dir),
    );
    script.push_str(&format!(
        " -fq1 {} -t {} -f {}",
        shell_quote_path(r1),
        threads.max(1),
        shell_quote_str(&adapter_r1),
    ));
    if let Some(r2) = r2 {
        script.push_str(&format!(
            " -fq2 {} -s {}",
            shell_quote_path(r2),
            shell_quote_str(&adapter_r2),
        ));
    }
    script.push_str(&format!(" -fqo leehom --log {}", shell_quote_path(&log_path),));
    script.push('\n');
    if let Some(output_r2) = output_r2 {
        script.push_str(&format!(
            "mv {} {}\n",
            shell_quote_str("leehom_r1.fq.gz"),
            shell_quote_path(output_r1),
        ));
        script.push_str(&format!(
            "mv {} {}\n",
            shell_quote_str("leehom_r2.fq.gz"),
            shell_quote_path(output_r2),
        ));
        script.push_str(&format!(
            "rm -f {} {} {} {}\n",
            shell_quote_str("leehom.fq.gz"),
            shell_quote_str("leehom.fail.fq.gz"),
            shell_quote_str("leehom_r1.fail.fq.gz"),
            shell_quote_str("leehom_r2.fail.fq.gz"),
        ));
    } else {
        script.push_str(&format!(
            "mv {} {}\n",
            shell_quote_str("leehom.fq.gz"),
            shell_quote_path(output_r1),
        ));
        script.push_str(&format!(
            "rm -f {} {} {}\n",
            shell_quote_str("leehom.fail.fq.gz"),
            shell_quote_str("leehom_r1.fail.fq.gz"),
            shell_quote_str("leehom_r2.fail.fq.gz"),
        ));
    }
    script.push_str(&write_trim_report_script(
        "leehom",
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        adapter_bank,
        None,
        None,
        options,
        None,
        None,
    ));
    Ok(wrap_trim_shell_script_with_report(
        &script,
        output_r1,
        output_r2,
        report_json,
        Some(log_path.as_path()),
    ))
}

fn adapterremoval_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let mut command = vec![
        "adapterremoval".to_string(),
        "--threads".to_string(),
        threads.max(1).to_string(),
        "--file1".to_string(),
        r1.display().to_string(),
        "--output1".to_string(),
        output_r1.display().to_string(),
    ];
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        command.extend([
            "--file2".to_string(),
            r2.display().to_string(),
            "--output2".to_string(),
            output_r2.display().to_string(),
            "--singleton".to_string(),
            "/dev/null".to_string(),
        ]);
    }
    command.extend(["--discarded".to_string(), "/dev/null".to_string()]);
    if matches!(options.resolved_adapter_policy().as_str(), "bank" | "ancient_strict") {
        let adapters = enabled_adapter_sequences(adapter_bank);
        if let Some(adapter_1) = adapters.first() {
            command.extend(["--adapter1".to_string(), adapter_1.clone()]);
            command.extend([
                "--adapter2".to_string(),
                adapters.get(1).cloned().unwrap_or_else(|| adapter_1.clone()),
            ]);
        }
    }
    if let Some(min_length) = options.min_length {
        command.extend(["--minlength".to_string(), min_length.to_string()]);
    }
    if let Some(quality_cutoff) = options.quality_cutoff {
        command.push("--trimqualities".to_string());
        command.extend(["--minquality".to_string(), quality_cutoff.to_string()]);
    }
    Ok(wrap_trim_command_with_report(
        "adapterremoval",
        &command,
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        adapter_bank,
        None,
        None,
        options,
        None,
        None,
    ))
}

fn alientrimmer_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let output_dir = output_r1
        .parent()
        .ok_or_else(|| anyhow!("alientrimmer output path must have a parent directory"))?;
    let adapter_file = output_dir.join("alientrimmer_adapters.txt");
    let adapter_sequences = match options.resolved_adapter_policy().as_str() {
        "bank" | "ancient_strict" => enabled_adapter_sequences(adapter_bank),
        _ => vec![fallback_trim_adapter_sequence(0).to_string()],
    };
    let adapter_payload = adapter_sequences.join("\n");
    let mut script = format!(
        "set -eu\nmkdir -p {}\ncat > {} <<'EOF'\n{}\nEOF\nalientrimmer",
        shell_quote_path(output_dir),
        shell_quote_path(&adapter_file),
        adapter_payload,
    );
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        script.push_str(&format!(
            " -if {} -ir {} -c {} -of {} -or {}",
            shell_quote_path(r1),
            shell_quote_path(r2),
            shell_quote_path(&adapter_file),
            shell_quote_path(output_r1),
            shell_quote_path(output_r2),
        ));
    } else {
        script.push_str(&format!(
            " -i {} -c {} -o {}",
            shell_quote_path(r1),
            shell_quote_path(&adapter_file),
            shell_quote_path(output_r1),
        ));
    }
    if let Some(min_length) = options.min_length {
        script.push_str(&format!(" -l {min_length}"));
    }
    if let Some(quality_cutoff) = options.quality_cutoff {
        script.push_str(&format!(" -q {quality_cutoff}"));
    }
    if output_r1.extension().is_some_and(|ext| ext == "gz") {
        script.push_str(" -z");
    }
    script.push('\n');
    script.push_str(&write_trim_report_script(
        "alientrimmer",
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        adapter_bank,
        None,
        None,
        options,
        None,
        None,
    ));
    Ok(vec!["sh".to_string(), "-lc".to_string(), script])
}

fn trimmomatic_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let mut command = vec!["trimmomatic".to_string()];
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        let output_dir = output_r1
            .parent()
            .ok_or_else(|| anyhow!("trimmomatic output path must have a parent directory"))?;
        let unpaired_r1 = output_dir.join("R1.trimmomatic.unpaired.fastq.gz");
        let unpaired_r2 = output_dir.join("R2.trimmomatic.unpaired.fastq.gz");
        command.extend([
            "PE".to_string(),
            "-threads".to_string(),
            threads.max(1).to_string(),
            "-phred33".to_string(),
            r1.display().to_string(),
            r2.display().to_string(),
            output_r1.display().to_string(),
            unpaired_r1.display().to_string(),
            output_r2.display().to_string(),
            unpaired_r2.display().to_string(),
        ]);
    } else {
        command.extend([
            "SE".to_string(),
            "-threads".to_string(),
            threads.max(1).to_string(),
            "-phred33".to_string(),
            r1.display().to_string(),
            output_r1.display().to_string(),
        ]);
    }
    if let Some(quality_cutoff) = options.quality_cutoff {
        command.push(format!("SLIDINGWINDOW:4:{quality_cutoff}"));
    }
    if let Some(min_length) = options.min_length {
        command.push(format!("MINLEN:{min_length}"));
    }
    Ok(wrap_trim_command_with_report(
        "trimmomatic",
        &command,
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        None,
        None,
        None,
        options,
        None,
        None,
    ))
}

fn trim_galore_command_template(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
) -> Result<Vec<String>> {
    let output_dir = output_r1
        .parent()
        .ok_or_else(|| anyhow!("trim_galore output path must have a parent directory"))?;
    let working_dir = output_dir.join("trim_galore_run");
    let mut script = format!(
        "set -eu\nmkdir -p {}\ntrim_galore --output_dir {} --cores {}",
        shell_quote_path(&working_dir),
        shell_quote_path(&working_dir),
        threads.max(1),
    );
    if let Some(min_length) = options.min_length {
        script.push_str(&format!(" --length {min_length}"));
    }
    if let Some(quality_cutoff) = options.quality_cutoff {
        script.push_str(&format!(" -q {quality_cutoff}"));
    } else {
        // Trim Galore defaults to quality trimming at Q20 unless it is overridden explicitly.
        script.push_str(" -q 0");
    }
    match options.resolved_adapter_policy().as_str() {
        "none" => {
            // Trim Galore defaults to adapter auto-detection and falls back to Illumina adapters.
            // Pin a no-op adapter sequence so the governed bank-free cohort stays semantically honest.
            script.push_str(" --adapter X");
        }
        "bank" | "ancient_strict" => {
            if let Some(adapter_sequence) = enabled_adapter_sequences(adapter_bank).first() {
                script.push_str(&format!(" --adapter {}", shell_quote_str(adapter_sequence)));
            }
        }
        _ => {}
    }
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
    script.push_str(&move_trim_galore_output_script(
        &trim_galore_output_paths(&working_dir, r1, r2.is_some(), 1),
        output_r1,
    ));
    if let (Some(r2), Some(output_r2)) = (r2, output_r2) {
        script.push_str(&move_trim_galore_output_script(
            &trim_galore_output_paths(&working_dir, r2, true, 2),
            output_r2,
        ));
    }
    script.push_str(&write_trim_report_script(
        "trim_galore",
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        adapter_bank,
        None,
        None,
        options,
        None,
        None,
    ));
    Ok(vec!["sh".to_string(), "-lc".to_string(), script])
}

fn move_trim_galore_output_script(candidates: &[PathBuf], output_path: &Path) -> String {
    let mut script = String::from("trim_galore_output_moved=0\n");
    for candidate in candidates {
        script.push_str(&format!(
            "if [ -f {} ]; then mv {} {}; trim_galore_output_moved=1; fi\n",
            shell_quote_path(candidate),
            shell_quote_path(candidate),
            shell_quote_path(output_path),
        ));
    }
    script.push_str(
        "[ \"$trim_galore_output_moved\" = 1 ] || { echo 'trim_galore did not produce an expected output file' >&2; exit 1; }\n",
    );
    script
}

fn trim_galore_output_paths(
    output_dir: &Path,
    reads: &Path,
    paired_end: bool,
    mate_index: u8,
) -> Vec<PathBuf> {
    let file_name = reads.file_name().and_then(|name| name.to_str()).unwrap_or("reads.fastq.gz");
    let candidate_names = if let Some(stripped) = file_name.strip_suffix(".fastq.gz") {
        if paired_end {
            vec![format!("{stripped}_val_{mate_index}.fq.gz"), format!("{stripped}_trimmed.fq.gz")]
        } else {
            vec![format!("{stripped}_trimmed.fq.gz")]
        }
    } else if let Some(stripped) = file_name.strip_suffix(".fq.gz") {
        if paired_end {
            vec![format!("{stripped}_val_{mate_index}.fq.gz"), format!("{stripped}_trimmed.fq.gz")]
        } else {
            vec![format!("{stripped}_trimmed.fq.gz")]
        }
    } else if let Some(stripped) = file_name.strip_suffix(".fastq") {
        if paired_end {
            vec![format!("{stripped}_val_{mate_index}.fq"), format!("{stripped}_trimmed.fq")]
        } else {
            vec![format!("{stripped}_trimmed.fq")]
        }
    } else if let Some(stripped) = file_name.strip_suffix(".fq") {
        if paired_end {
            vec![format!("{stripped}_val_{mate_index}.fq"), format!("{stripped}_trimmed.fq")]
        } else {
            vec![format!("{stripped}_trimmed.fq")]
        }
    } else {
        vec![format!("{file_name}_trimmed.fq.gz")]
    };
    candidate_names.into_iter().map(|name| output_dir.join(name)).collect()
}

fn enabled_adapter_sequences(adapter_bank: Option<&serde_json::Value>) -> Vec<String> {
    adapter_bank
        .and_then(|bank| bank.get("enabled_entries"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            entry.get("sequence").and_then(serde_json::Value::as_str).map(str::to_string)
        })
        .collect()
}

fn fallback_trim_adapter_sequence(index: usize) -> &'static str {
    if index == 0 {
        FALLBACK_TRIM_ADAPTER_R1
    } else {
        FALLBACK_TRIM_ADAPTER_R2
    }
}

fn resolved_adapter_pair(adapter_bank: Option<&serde_json::Value>) -> (String, String) {
    let adapter_sequences = enabled_adapter_sequences(adapter_bank);
    let adapter_r1 = adapter_sequences
        .first()
        .cloned()
        .unwrap_or_else(|| fallback_trim_adapter_sequence(0).to_string());
    let adapter_r2 = adapter_sequences
        .get(1)
        .cloned()
        .unwrap_or_else(|| fallback_trim_adapter_sequence(1).to_string());
    (adapter_r1, adapter_r2)
}

fn shell_join(command: &[String]) -> String {
    command.iter().map(|part| shell_quote_str(part)).collect::<Vec<_>>().join(" ")
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
        adapterremoval_command_template, alientrimmer_command_template, atropos_command_template,
        fastx_clipper_command_template, leehom_trim_command_template, skewer_trim_command_template,
        trim_galore_command_template, TrimPlanOptions, FALLBACK_TRIM_ADAPTER_R1,
        FALLBACK_TRIM_ADAPTER_R2,
    };
    use std::path::Path;

    #[test]
    fn adapterremoval_trim_redirects_undeclared_side_outputs() {
        let command = adapterremoval_command_template(
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out/R1.adapterremoval.fastq.gz"),
            Some(Path::new("out/R2.adapterremoval.fastq.gz")),
            Path::new("out/trim_report.json"),
            1,
            None,
            &TrimPlanOptions { min_length: Some(30), ..TrimPlanOptions::default() },
        )
        .expect("adapterremoval command");

        let script = command.get(2).expect("shell script");
        assert!(script.contains("'--singleton' '/dev/null'"));
        assert!(script.contains("'--discarded' '/dev/null'"));
    }

    #[test]
    fn atropos_trim_requires_full_overlap_for_fallback_adapter_when_adapter_policy_is_none() {
        let command = atropos_command_template(
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out/R1.atropos.fastq.gz"),
            Some(Path::new("out/R2.atropos.fastq.gz")),
            Path::new("out/trim_report.json"),
            1,
            None,
            &TrimPlanOptions { min_length: Some(30), ..TrimPlanOptions::default() },
        )
        .expect("atropos command");

        let script = command.get(2).expect("shell script");
        assert!(script.contains("mkdir -p 'out'"));
        assert!(script.contains(&format!("'-a' '{}'", FALLBACK_TRIM_ADAPTER_R1)));
        assert!(script.contains(&format!("'-A' '{}'", FALLBACK_TRIM_ADAPTER_R2)));
        assert!(script.contains(&format!("'-O' '{}'", FALLBACK_TRIM_ADAPTER_R1.len())));
    }

    #[test]
    fn trim_galore_trim_disables_default_quality_and_adapter_trimming_for_bank_free_runs() {
        let command = trim_galore_command_template(
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out/R1.trimmed_trimmed.fq.gz"),
            Some(Path::new("out/R2.trimmed_trimmed.fq.gz")),
            Path::new("out/trim_report.json"),
            1,
            None,
            &TrimPlanOptions { min_length: Some(30), ..TrimPlanOptions::default() },
        )
        .expect("trim_galore command");

        let script = command.get(2).expect("shell script");
        assert!(script.contains(" -q 0"));
        assert!(script.contains(" --adapter X"));
        assert!(!script.contains(" --adapter2 "));
        assert!(!script.contains("AGATCGGAAGAGC"));
    }

    #[test]
    fn alientrimmer_trim_materializes_adapter_file_and_explicit_outputs() {
        let command = alientrimmer_command_template(
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out/R1.alientrimmer.fastq.gz"),
            Some(Path::new("out/R2.alientrimmer.fastq.gz")),
            Path::new("out/trim_report.json"),
            8,
            None,
            &TrimPlanOptions {
                min_length: Some(30),
                quality_cutoff: Some(13),
                ..TrimPlanOptions::default()
            },
        )
        .expect("alientrimmer command");

        let script = command.get(2).expect("shell script");
        assert!(script.contains("alientrimmer_adapters.txt"));
        assert!(script.contains(&format!("\n{}\nEOF\nalientrimmer", FALLBACK_TRIM_ADAPTER_R1)));
        assert!(script.contains(" -if 'reads_R1.fastq.gz'"));
        assert!(script.contains(" -ir 'reads_R2.fastq.gz'"));
        assert!(script.contains(" -of 'out/R1.alientrimmer.fastq.gz'"));
        assert!(script.contains(" -or 'out/R2.alientrimmer.fastq.gz'"));
        assert!(script.contains(" -l 30"));
        assert!(script.contains(" -q 13"));
        assert!(script.contains(" -z"));
    }

    #[test]
    fn fastx_clipper_trim_executes_both_mates_with_explicit_outputs() {
        let command = fastx_clipper_command_template(
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out/R1.fastx_clipper.fastq.gz"),
            Some(Path::new("out/R2.fastx_clipper.fastq.gz")),
            Path::new("out/trim_report.json"),
            8,
            None,
            &TrimPlanOptions::default(),
        )
        .expect("fastx_clipper command");

        let script = command.get(2).expect("shell script");
        assert!(script.contains("mkdir -p 'out'"));
        assert!(script.contains(&format!(
            "gzip -dc 'reads_R1.fastq.gz' | fastx_clipper -Q33 -a '{}' -o 'out/R1.fastx_clipper.fastq.gz' -M {} -z",
            FALLBACK_TRIM_ADAPTER_R1,
            FALLBACK_TRIM_ADAPTER_R1.len()
        )));
        assert!(script.contains(&format!(
            "gzip -dc 'reads_R2.fastq.gz' | fastx_clipper -Q33 -a '{}' -o 'out/R2.fastx_clipper.fastq.gz' -M {} -z",
            FALLBACK_TRIM_ADAPTER_R1,
            FALLBACK_TRIM_ADAPTER_R1.len()
        )));
    }

    #[test]
    fn skewer_trim_materializes_paired_outputs_from_named_prefix() {
        let command = skewer_trim_command_template(
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out/R1.skewer.fastq.gz"),
            Some(Path::new("out/R2.skewer.fastq.gz")),
            Path::new("out/trim_report.json"),
            8,
            None,
            &TrimPlanOptions::default(),
        )
        .expect("skewer command");

        let script = command.get(2).expect("shell script");
        assert!(script.contains(&format!(
            "skewer -m pe -t 8 -o 'out/skewer' -x '{}' -y '{}' -z 'reads_R1.fastq.gz' 'reads_R2.fastq.gz'",
            FALLBACK_TRIM_ADAPTER_R1,
            FALLBACK_TRIM_ADAPTER_R2
        )));
        assert!(script.contains("skewer-trimmed-pair1.fastq.gz"));
        assert!(script.contains("skewer-trimmed-pair2.fastq.gz"));
    }

    #[test]
    fn leehom_trim_routes_trimmed_pair_outputs_explicitly() {
        let command = leehom_trim_command_template(
            Path::new("reads_R1.fastq.gz"),
            Some(Path::new("reads_R2.fastq.gz")),
            Path::new("out/R1.leehom.fastq.gz"),
            Some(Path::new("out/R2.leehom.fastq.gz")),
            Path::new("out/trim_report.json"),
            8,
            None,
            &TrimPlanOptions::default(),
        )
        .expect("leehom command");

        let script = command.get(2).expect("shell script");
        assert!(script.contains(&format!(
            "cd 'out'\nleehom -fq1 'reads_R1.fastq.gz' -t 8 -f '{}' -fq2 'reads_R2.fastq.gz' -s '{}'",
            FALLBACK_TRIM_ADAPTER_R1, FALLBACK_TRIM_ADAPTER_R2
        )));
        assert!(script.contains("-fqo leehom --log 'out/trim_report.leehom.log'"));
        assert!(script.contains("mv 'leehom_r1.fq.gz' 'out/R1.leehom.fastq.gz'"));
        assert!(script.contains("mv 'leehom_r2.fq.gz' 'out/R2.leehom.fastq.gz'"));
        assert!(script.contains("rm -f 'leehom.fq.gz' 'leehom.fail.fq.gz' 'leehom_r1.fail.fq.gz' 'leehom_r2.fail.fq.gz'"));
        assert!(script.contains("--log 'out/trim_report.leehom.log'"));
    }
}
