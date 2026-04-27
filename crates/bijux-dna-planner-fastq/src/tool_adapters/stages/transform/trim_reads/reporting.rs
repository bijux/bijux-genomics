use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use bijux_dna_core::prelude::{ArtifactId, ArtifactRole};
use bijux_dna_domain_fastq::TRIM_READS_REPORT_SCHEMA_VERSION;
use bijux_dna_stage_contract::ArtifactRef;

use super::{shell_join, shell_quote_path, shell_quote_str, PairedMode, TrimPlanOptions, STAGE_ID};

pub(super) fn raw_backend_report_path(
    report_json: &Path,
    tool_id: &str,
    extension: &str,
) -> PathBuf {
    report_json.with_file_name(format!("trim_report.{tool_id}.{extension}"))
}

pub(super) fn trim_raw_backend_output(tool_id: &str, report_json: &Path) -> Option<ArtifactRef> {
    match tool_id {
        "fastp" | "cutadapt" => Some(ArtifactRef::optional(
            ArtifactId::from_static("raw_backend_report_json"),
            raw_backend_report_path(report_json, tool_id, "json"),
            ArtifactRole::ReportJson,
        )),
        "bbduk" => Some(ArtifactRef::optional(
            ArtifactId::from_static("raw_backend_report_txt"),
            raw_backend_report_path(report_json, tool_id, "stats.txt"),
            ArtifactRole::Log,
        )),
        _ => None,
    }
}

fn report_context_string(context: Option<&serde_json::Value>, key: &str) -> Option<String> {
    context
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn governed_trim_report_payload(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
    raw_backend_report: Option<&Path>,
    raw_backend_report_format: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": TRIM_READS_REPORT_SCHEMA_VERSION,
        "stage": STAGE_ID.as_str(),
        "stage_id": STAGE_ID.as_str(),
        "tool_id": tool_id,
        "paired_mode": if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        "threads": threads,
        "input_r1": r1.display().to_string(),
        "input_r2": r2.map(|path| path.display().to_string()),
        "output_r1": output_r1.display().to_string(),
        "output_r2": output_r2.map(|path| path.display().to_string()),
        "min_length": options.resolved_min_length(),
        "quality_cutoff": options.quality_cutoff,
        "adapter_policy": options.resolved_adapter_policy(),
        "polyx_policy": options.resolved_polyx_policy(),
        "n_policy": options.resolved_n_policy(),
        "contaminant_policy": options.resolved_contaminant_policy(),
        "adapter_bank_id": report_context_string(adapter_bank, "bank_id"),
        "adapter_bank_hash": report_context_string(adapter_bank, "bank_hash"),
        "adapter_preset": report_context_string(adapter_bank, "preset"),
        "adapter_overrides": adapter_bank.and_then(|context| context.get("adapter_selection").cloned()),
        "polyx_bank_id": report_context_string(polyx_bank, "bank_id"),
        "polyx_bank_hash": report_context_string(polyx_bank, "bank_hash"),
        "polyx_preset": report_context_string(polyx_bank, "preset"),
        "contaminant_bank_id": report_context_string(contaminant_bank, "bank_id"),
        "contaminant_bank_hash": report_context_string(contaminant_bank, "bank_hash"),
        "contaminant_preset": report_context_string(contaminant_bank, "preset"),
        "reads_in": serde_json::Value::Null,
        "reads_out": serde_json::Value::Null,
        "bases_in": serde_json::Value::Null,
        "bases_out": serde_json::Value::Null,
        "pairs_in": serde_json::Value::Null,
        "pairs_out": serde_json::Value::Null,
        "mean_q_before": serde_json::Value::Null,
        "mean_q_after": serde_json::Value::Null,
        "runtime_s": serde_json::Value::Null,
        "memory_mb": serde_json::Value::Null,
        "raw_backend_report": raw_backend_report.map(|path| path.display().to_string()),
        "raw_backend_report_format": raw_backend_report_format,
    })
}

pub(super) fn write_trim_report_script(
    tool_id: &str,
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
    raw_backend_report: Option<&Path>,
    raw_backend_report_format: Option<&str>,
) -> String {
    let payload = governed_trim_report_payload(
        tool_id,
        r1,
        r2,
        output_r1,
        output_r2,
        threads,
        adapter_bank,
        polyx_bank,
        contaminant_bank,
        options,
        raw_backend_report,
        raw_backend_report_format,
    );
    format!(
        "printf '%s\\n' {} > {}\n",
        shell_quote_str(&payload.to_string()),
        shell_quote_path(report_json),
    )
}

pub(super) fn wrap_trim_command_with_report(
    tool_id: &str,
    command: &[String],
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    threads: u32,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
    raw_backend_report: Option<&Path>,
    raw_backend_report_format: Option<&str>,
) -> Vec<String> {
    let mut script = format!("set -eu\n{}\n", shell_join(command));
    script.push_str(&write_trim_report_script(
        tool_id,
        r1,
        r2,
        output_r1,
        output_r2,
        report_json,
        threads,
        adapter_bank,
        polyx_bank,
        contaminant_bank,
        options,
        raw_backend_report,
        raw_backend_report_format,
    ));
    wrap_trim_shell_script_with_report(
        &script,
        output_r1,
        output_r2,
        report_json,
        raw_backend_report,
    )
}

pub(super) fn wrap_trim_shell_script_with_report(
    script: &str,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    raw_backend_report: Option<&Path>,
) -> Vec<String> {
    let mut dir_paths = BTreeSet::<PathBuf>::new();
    for path in
        [Some(output_r1), output_r2, Some(report_json), raw_backend_report].into_iter().flatten()
    {
        if let Some(parent) = path.parent() {
            dir_paths.insert(parent.to_path_buf());
        }
    }
    let mut wrapped = String::from("set -eu\n");
    if !dir_paths.is_empty() {
        wrapped.push_str("mkdir -p");
        for path in dir_paths {
            wrapped.push(' ');
            wrapped.push_str(&shell_quote_path(&path));
        }
        wrapped.push('\n');
    }
    wrapped.push_str(script.strip_prefix("set -eu\n").unwrap_or(script));
    vec!["sh".to_string(), "-lc".to_string(), wrapped]
}

pub(super) fn move_first_existing_output_script(
    candidates: &[PathBuf],
    output_path: &Path,
    label: &str,
) -> String {
    let mut script = String::from("trim_output_moved=0\n");
    for candidate in candidates {
        script
            .write_fmt(format_args!(
                "if [ -f {} ]; then mv {} {}; trim_output_moved=1; fi\n",
                shell_quote_path(candidate),
                shell_quote_path(candidate),
                shell_quote_path(output_path),
            ))
            .unwrap_or_else(|_| unreachable!("writing to String cannot fail"));
    }
    script
        .write_fmt(format_args!(
            "[ \"$trim_output_moved\" = 1 ] || {{ echo '{}' >&2; exit 1; }}\n",
            label.replace('\'', "\"")
        ))
        .unwrap_or_else(|_| unreachable!("writing to String cannot fail"));
    script
}
