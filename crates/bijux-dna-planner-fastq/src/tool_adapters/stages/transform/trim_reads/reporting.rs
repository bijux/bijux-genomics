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

pub(super) fn trim_raw_backend_output_path(tool_id: &str, out_dir: &Path) -> Option<PathBuf> {
    let report_json = out_dir.join("trim_report.json");
    match tool_id {
        "fastp" | "cutadapt" => Some(raw_backend_report_path(&report_json, tool_id, "json")),
        "bbduk" => Some(raw_backend_report_path(&report_json, tool_id, "stats.txt")),
        _ => None,
    }
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

fn report_context_vec(context: Option<&serde_json::Value>, key: &str) -> Vec<String> {
    context
        .and_then(|value| value.get(key))
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn prepared_adapter_bank_report(
    adapter_bank: Option<&serde_json::Value>,
) -> Option<serde_json::Value> {
    let context = adapter_bank?;
    let enabled_adapter_ids = context
        .get("enabled_entries")
        .and_then(serde_json::Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.get("id").and_then(serde_json::Value::as_str))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(serde_json::json!({
        "schema_version": "bijux.fastq.prepare_adapter_bank.report.v1",
        "stage": "fastq.prepare_adapter_bank",
        "stage_id": "fastq.prepare_adapter_bank",
        "tool_id": "bijux",
        "bank_id": report_context_string(adapter_bank, "bank_id"),
        "bank_version": report_context_string(adapter_bank, "bank_version").unwrap_or_else(|| "unknown".to_string()),
        "bank_hash": report_context_string(adapter_bank, "bank_hash"),
        "presets_hash": report_context_string(adapter_bank, "presets_hash"),
        "preset": report_context_string(adapter_bank, "preset"),
        "preset_hash": report_context_string(adapter_bank, "preset_hash"),
        "enabled_categories": report_context_vec(adapter_bank, "enabled_categories"),
        "disabled_categories": report_context_vec(adapter_bank, "disabled_categories"),
        "enable_adapters": report_context_vec(adapter_bank, "enable_adapters"),
        "disable_adapters": report_context_vec(adapter_bank, "disable_adapters"),
        "enabled_adapter_ids": enabled_adapter_ids,
    }))
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
    let effective_trim_params = serde_json::json!({
        "threads": threads,
        "min_length": options.resolved_min_length(),
        "quality_cutoff": options.quality_cutoff,
        "adapter_policy": options.resolved_adapter_policy(),
        "polyx_policy": options.resolved_polyx_policy(),
        "n_policy": options.resolved_n_policy(),
        "contaminant_policy": options.resolved_contaminant_policy(),
    });
    let prepared_adapter_bank = prepared_adapter_bank_report(adapter_bank);
    let mut payload = serde_json::Map::new();
    payload.insert(
        "schema_version".to_string(),
        serde_json::Value::String(TRIM_READS_REPORT_SCHEMA_VERSION.to_string()),
    );
    payload.insert("stage".to_string(), serde_json::Value::String(STAGE_ID.as_str().to_string()));
    payload
        .insert("stage_id".to_string(), serde_json::Value::String(STAGE_ID.as_str().to_string()));
    payload.insert("tool_id".to_string(), serde_json::Value::String(tool_id.to_string()));
    payload.insert(
        "paired_mode".to_string(),
        serde_json::to_value(if r2.is_some() {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        })
        .unwrap_or(serde_json::Value::Null),
    );
    payload.insert("threads".to_string(), serde_json::json!(threads));
    payload.insert("trimming_backend".to_string(), serde_json::Value::String(tool_id.to_string()));
    payload.insert(
        "backend_mode".to_string(),
        serde_json::Value::String(super::trim_backend_mode(tool_id).to_string()),
    );
    payload.insert("input_r1".to_string(), serde_json::json!(r1.display().to_string()));
    payload.insert(
        "input_r2".to_string(),
        serde_json::json!(r2.map(|path| path.display().to_string())),
    );
    payload.insert("output_r1".to_string(), serde_json::json!(output_r1.display().to_string()));
    payload.insert(
        "output_r2".to_string(),
        serde_json::json!(output_r2.map(|path| path.display().to_string())),
    );
    payload.insert("min_length".to_string(), serde_json::json!(options.resolved_min_length()));
    payload.insert("quality_cutoff".to_string(), serde_json::json!(options.quality_cutoff));
    payload
        .insert("adapter_policy".to_string(), serde_json::json!(options.resolved_adapter_policy()));
    payload.insert("polyx_policy".to_string(), serde_json::json!(options.resolved_polyx_policy()));
    payload.insert("n_policy".to_string(), serde_json::json!(options.resolved_n_policy()));
    payload.insert(
        "contaminant_policy".to_string(),
        serde_json::json!(options.resolved_contaminant_policy()),
    );
    payload.insert(
        "adapter_bank_id".to_string(),
        serde_json::json!(report_context_string(adapter_bank, "bank_id")),
    );
    payload.insert(
        "adapter_bank_hash".to_string(),
        serde_json::json!(report_context_string(adapter_bank, "bank_hash")),
    );
    payload.insert(
        "adapter_preset".to_string(),
        serde_json::json!(report_context_string(adapter_bank, "preset")),
    );
    payload.insert(
        "detected_adapter_source".to_string(),
        serde_json::json!(report_context_string(adapter_bank, "preset")
            .map(|preset| format!("prepared_adapter_bank:{preset}"))),
    );
    payload.insert(
        "adapter_overrides".to_string(),
        adapter_bank
            .and_then(|context| context.get("adapter_selection").cloned())
            .unwrap_or(serde_json::Value::Null),
    );
    payload.insert(
        "prepared_adapter_bank".to_string(),
        prepared_adapter_bank.unwrap_or(serde_json::Value::Null),
    );
    payload.insert(
        "polyx_bank_id".to_string(),
        serde_json::json!(report_context_string(polyx_bank, "bank_id")),
    );
    payload.insert(
        "polyx_bank_hash".to_string(),
        serde_json::json!(report_context_string(polyx_bank, "bank_hash")),
    );
    payload.insert(
        "polyx_preset".to_string(),
        serde_json::json!(report_context_string(polyx_bank, "preset")),
    );
    payload.insert(
        "contaminant_bank_id".to_string(),
        serde_json::json!(report_context_string(contaminant_bank, "bank_id")),
    );
    payload.insert(
        "contaminant_bank_hash".to_string(),
        serde_json::json!(report_context_string(contaminant_bank, "bank_hash")),
    );
    payload.insert(
        "contaminant_preset".to_string(),
        serde_json::json!(report_context_string(contaminant_bank, "preset")),
    );
    for field in [
        "reads_in",
        "reads_out",
        "bases_in",
        "bases_out",
        "pairs_in",
        "pairs_out",
        "mean_q_before",
        "mean_q_after",
        "runtime_s",
        "memory_mb",
    ] {
        payload.insert(field.to_string(), serde_json::Value::Null);
    }
    payload.insert("effective_trim_params".to_string(), effective_trim_params);
    payload.insert(
        "raw_backend_report".to_string(),
        serde_json::json!(raw_backend_report.map(|path| path.display().to_string())),
    );
    payload.insert(
        "raw_backend_report_format".to_string(),
        serde_json::json!(raw_backend_report_format),
    );
    serde_json::Value::Object(payload)
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
