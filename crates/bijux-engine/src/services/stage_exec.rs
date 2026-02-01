use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_env_runtime::api::{ResolvedImage, RunnerKind};
use chrono::Utc;
use flate2::read::GzDecoder;
use tracing::info;
use uuid::Uuid;

use crate::api::{
    cleanup_execution, execution_memory_mb, hash_file_sha256, run_filter_execution,
    run_merge_execution, run_multiqc_execution, run_tool_execution, run_validate_execution,
};
use crate::services::observer::Observer;
use crate::services::run_artifacts::{
    default_trace_ids, params_hash, run_artifacts_dir_for_out,
    write_effective_adapters_from_provenance, write_facts_jsonl, write_filter_report_v1,
    write_merge_report_v1, write_metrics_envelope, write_observability_manifest,
    write_plan_artifacts, write_progress_event_jsonl, write_qc_post_report_v1,
    write_retention_report_v1, write_runs_export_jsonl, write_stage_event_jsonl,
    write_stage_metrics_json, write_stage_report_v1, write_telemetry_event,
    write_tool_invocation_json, write_trim_report_v1, write_validate_report_v1,
};
use bijux_core::run_index::{insert_stage_row, StageIndexRow};
use bijux_core::{
    parameters_json_canonicalization, AdapterBankProvenanceV1, BankRefV1, FactsRowV1,
    FastqCorrectMetricsV1, FastqDeltaMetricsV1, FastqFilterMetricsV1, FastqMergeMetricsV1,
    FastqPreprocessMetricsV1, FastqQcPostMetricsV1, FastqTrimMetricsV1, FastqUmiMetricsV1,
    FastqValidateMetricsV1, MetricContextV1, RetentionReportMetricV1, StageMetricsV1,
    StageObservabilityContextV1, StagePlanV1, ToolInvocationV1,
};
use bijux_domain_fastq::{evaluate_invariants, parse_effective_params, thresholds_from_env};

#[derive(Debug, Clone)]
pub struct StageResultV1 {
    pub run_id: String,
    pub exit_code: i32,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub outputs: Vec<PathBuf>,
    pub metrics_path: Option<PathBuf>,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

#[derive(Debug)]
struct ExecutionEnvelope {
    exit_code: i32,
    stdout: String,
    stderr: String,
    command: String,
}

fn resolved_image_for_plan(
    image: &bijux_core::ContainerImageRefV1,
    runner: RunnerKind,
) -> ResolvedImage {
    ResolvedImage {
        full_name: image.image.clone(),
        arch: "unknown".to_string(),
        runner,
    }
}

fn adapter_bank_from_params(params: &serde_json::Value) -> Option<AdapterBankProvenanceV1> {
    params
        .get("adapter_bank")
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn banks_from_params(params: &serde_json::Value) -> Option<serde_json::Value> {
    let mut banks = serde_json::Map::new();
    for (key, field) in [
        ("adapter", "adapter_bank"),
        ("polyx", "polyx_bank"),
        ("contaminant", "contaminant_bank"),
    ] {
        if let Some(value) = params.get(field) {
            banks.insert(key.to_string(), value.clone());
        }
    }
    if banks.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(banks))
    }
}

fn metric_context_from_params(
    plan: &StagePlanV1,
    runner: RunnerKind,
    input_hash: &str,
    params_hash: &str,
    params: &serde_json::Value,
) -> MetricContextV1 {
    let mut presets = BTreeMap::new();
    let mut banks = BTreeMap::new();
    for (key, field) in [
        ("adapter", "adapter_bank"),
        ("polyx", "polyx_bank"),
        ("contaminant", "contaminant_bank"),
    ] {
        if let Some(value) = params.get(field) {
            if let Some(preset) = value.get("preset").and_then(|v| v.as_str()) {
                presets.insert(key.to_string(), preset.to_string());
            }
            let bank_id = value.get("bank_id").and_then(|v| v.as_str());
            let bank_hash = value.get("bank_hash").and_then(|v| v.as_str());
            if let (Some(bank_id), Some(bank_hash)) = (bank_id, bank_hash) {
                banks.insert(
                    key.to_string(),
                    BankRefV1 {
                        bank_id: bank_id.to_string(),
                        bank_hash: bank_hash.to_string(),
                    },
                );
            }
        }
    }
    MetricContextV1 {
        tool_id: plan.tool_id.0.clone(),
        tool_version: plan.tool_version.clone(),
        image_digest: plan.image.digest.clone(),
        runner: runner.to_string(),
        platform: std::env::var("BIJUX_PLATFORM").unwrap_or_else(|_| "unknown".to_string()),
        input_hash: input_hash.to_string(),
        params_hash: params_hash.to_string(),
        presets,
        banks,
    }
}

#[derive(Debug, Clone)]
struct BankEntryRecord {
    id: String,
    sequence: String,
    rationale: String,
    source: String,
}

#[derive(Debug, Clone)]
struct BankReferenceRecord {
    id: String,
    file: String,
    sha256: String,
    rationale: String,
    source: String,
    fasta: Option<String>,
}

fn bank_entries_from_value(value: &serde_json::Value) -> Vec<BankEntryRecord> {
    value
        .get("enabled_entries")
        .and_then(serde_json::Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    Some(BankEntryRecord {
                        id: entry.get("id")?.as_str()?.to_string(),
                        sequence: entry.get("sequence")?.as_str()?.to_string(),
                        rationale: entry.get("rationale")?.as_str()?.to_string(),
                        source: entry.get("source")?.as_str()?.to_string(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn bank_references_from_value(value: &serde_json::Value) -> Vec<BankReferenceRecord> {
    value
        .get("references")
        .and_then(serde_json::Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    Some(BankReferenceRecord {
                        id: entry.get("id")?.as_str()?.to_string(),
                        file: entry.get("file")?.as_str()?.to_string(),
                        sha256: entry
                            .get("sha256")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        rationale: entry.get("rationale")?.as_str()?.to_string(),
                        source: entry.get("source")?.as_str()?.to_string(),
                        fasta: entry
                            .get("fasta")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn bank_refs_from_params(params: &serde_json::Value) -> serde_json::Value {
    let mut banks = serde_json::Map::new();
    for (key, field) in [
        ("adapter", "adapter_bank"),
        ("polyx", "polyx_bank"),
        ("contaminant", "contaminant_bank"),
    ] {
        if let Some(bank) = params.get(field) {
            let entry = serde_json::json!({
                "bank_id": bank.get("bank_id"),
                "bank_hash": bank.get("bank_hash"),
                "preset": bank.get("preset"),
                "preset_hash": bank.get("preset_hash"),
            });
            banks.insert(key.to_string(), entry);
        }
    }
    serde_json::Value::Object(banks)
}

fn retention_conditions_from_effective(
    stage_id: &str,
    effective_params: &serde_json::Value,
    raw_params: &serde_json::Value,
) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    let mut warning = None;
    if let Some(params) = parse_effective_params(stage_id, effective_params) {
        if let Some(map) = params.retention_conditions().as_object() {
            for (key, value) in map {
                out.insert(key.clone(), value.clone());
            }
        }
        out.insert("parameters".to_string(), effective_params.clone());
        out.insert(
            "condition".to_string(),
            serde_json::Value::String("effective".to_string()),
        );
    } else {
        warning = Some("effective_params_missing");
        out.insert("parameters".to_string(), raw_params.clone());
        out.insert(
            "condition".to_string(),
            serde_json::Value::String("unknown".to_string()),
        );
    }
    out.insert("banks".to_string(), bank_refs_from_params(raw_params));
    for key in [
        "min_len",
        "q",
        "max_n",
        "low_complexity_threshold",
        "kmer_ref",
        "merge_policy",
        "adapter_policy",
        "polyx_policy",
        "contaminant_policy",
    ] {
        out.entry(key.to_string())
            .or_insert(serde_json::Value::Null);
    }
    if let Some(flag) = warning {
        out.insert(
            "warning".to_string(),
            serde_json::Value::String(flag.to_string()),
        );
    }
    serde_json::Value::Object(out)
}

fn path_from_params(params: &serde_json::Value, key: &str) -> Option<PathBuf> {
    params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
}

fn find_fastqc_summary(dir: &Path) -> Option<PathBuf> {
    let direct = dir.join("summary.txt");
    if direct.exists() {
        return Some(direct);
    }
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let candidate = path.join("summary.txt");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

fn fastqc_modules_from_dir(dir: &Path) -> serde_json::Value {
    let Some(path) = find_fastqc_summary(dir) else {
        return serde_json::json!({});
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return serde_json::json!({});
    };
    let mut modules = serde_json::Map::new();
    for line in raw.lines() {
        let mut parts = line.split('\t');
        let Some(status) = parts.next() else {
            continue;
        };
        let Some(name) = parts.next() else {
            continue;
        };
        modules.insert(
            name.to_string(),
            serde_json::Value::String(status.to_string()),
        );
    }
    serde_json::Value::Object(modules)
}

fn find_fastqc_data(dir: &Path) -> Option<PathBuf> {
    let direct = dir.join("fastqc_data.txt");
    if direct.exists() {
        return Some(direct);
    }
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let candidate = path.join("fastqc_data.txt");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

fn adapter_suggestions_from_fastqc(dir: &Path) -> (serde_json::Value, Option<String>) {
    let Some(path) = find_fastqc_data(dir) else {
        return (serde_json::json!({}), None);
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return (serde_json::json!({}), None);
    };
    let mut in_module = false;
    let mut candidates = Vec::new();
    for line in raw.lines() {
        if line.starts_with(">>Overrepresented sequences") {
            in_module = true;
            continue;
        }
        if in_module && line.starts_with(">>END_MODULE") {
            break;
        }
        if !in_module || line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }
        let sequence = parts[0].to_string();
        let percent = parts[2].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
        let possible_source = parts.get(3).map(ToString::to_string);
        let confidence = if percent >= 1.0 {
            "high"
        } else if percent >= 0.1 {
            "medium"
        } else {
            "low"
        };
        let matched_preset = if sequence.contains("AGATCGGAAGAGC") {
            Some("illumina-default".to_string())
        } else if sequence.contains("CTGTCTCTTATA") || sequence.contains("TGGAATTCTCGG") {
            Some("ssdna".to_string())
        } else {
            None
        };
        candidates.push(serde_json::json!({
            "sequence": sequence,
            "percent": percent,
            "source": possible_source,
            "matched_preset": matched_preset,
            "confidence": confidence,
        }));
    }
    let suggested_preset = candidates
        .iter()
        .find_map(|entry| entry.get("matched_preset").and_then(|v| v.as_str()))
        .map(str::to_string);
    (
        serde_json::json!({
            "schema_version": "bijux.adapter_suggestions.v1",
            "candidates": candidates,
            "suggested_preset": suggested_preset,
        }),
        suggested_preset,
    )
}

fn parse_screen_report(path: &Path) -> (f64, serde_json::Value) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return (0.0, serde_json::json!({}));
    };
    let mut entries = Vec::new();
    let mut unmapped_percent = None;
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
            continue;
        }
        let label = parts[0].trim().to_string();
        let mut percent = None;
        for part in parts.iter().rev() {
            let part = part.trim_end_matches('%');
            if let Ok(value) = part.parse::<f64>() {
                percent = Some(value);
                break;
            }
        }
        if let Some(value) = percent {
            if label.to_lowercase().contains("no hit") || label.to_lowercase().contains("unmapped")
            {
                unmapped_percent = Some(value);
            }
            entries.push(serde_json::json!({
                "reference": label,
                "percent": value,
            }));
        }
    }
    let contamination_rate = unmapped_percent.map_or(0.0, |value| (100.0 - value).max(0.0) / 100.0);
    (
        contamination_rate,
        serde_json::json!({
            "schema_version": "bijux.screen_summary.v1",
            "entries": entries,
        }),
    )
}

fn tool_supports_polyx(tool_id: &str) -> bool {
    matches!(tool_id, "fastp")
}

fn tool_supports_kmer_filter(tool_id: &str) -> bool {
    matches!(tool_id, "bbduk")
}

fn polyx_unsupported_warning(tool_id: &str, params: &serde_json::Value) -> Option<String> {
    if params.get("polyx_bank").is_some() && !tool_supports_polyx(tool_id) {
        return Some(format!(
            "warning: polyx preset requested but tool '{tool_id}' does not advertise polyX support"
        ));
    }
    None
}

fn warnings_for_plan(plan: &StagePlanV1, params: &serde_json::Value) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some(msg) = polyx_unsupported_warning(plan.tool_id.0.as_str(), params) {
        warnings.push(msg);
    }
    if plan.stage_id.0 == "fastq.filter" {
        let redundant_filters = params
            .get("redundant_filters")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .collect::<Vec<&str>>()
            })
            .unwrap_or_default();
        if !redundant_filters.is_empty() {
            warnings.push(format!(
                "warning: filter stage received redundant filters already handled in trim: {}",
                redundant_filters.join(", ")
            ));
        }
    }
    if params.get("kmer_ref").is_some() && !tool_supports_kmer_filter(plan.tool_id.0.as_str()) {
        warnings.push(format!(
            "warning: k-mer filter requested but tool '{}' does not advertise k-mer support",
            plan.tool_id.0
        ));
    }
    if let Some(redundant) = params
        .get("redundant_filters")
        .and_then(|value| value.as_array())
    {
        if !redundant.is_empty() {
            let rendered: Vec<String> = redundant
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect();
            if !rendered.is_empty() {
                warnings.push(format!(
                    "warning: filter may be redundant; already handled by trim: {}",
                    rendered.join(", ")
                ));
            }
        }
    }
    warnings
}

fn quality_gate_decision(
    stage_id: &str,
    metrics: &serde_json::Value,
    reads_in: Option<u64>,
    reads_out: Option<u64>,
) -> Option<serde_json::Value> {
    if !matches!(
        stage_id,
        "fastq.trim" | "fastq.filter" | "fastq.qc_post" | "fastq.validate_pre"
    ) {
        return None;
    }
    let mut status = "pass".to_string();
    let mut reasons = Vec::new();
    let read_retention = metrics
        .get("delta_metrics")
        .and_then(|value| value.get("read_retention"))
        .and_then(serde_json::Value::as_f64)
        .or_else(|| {
            if let (Some(r_in), Some(r_out)) = (reads_in, reads_out) {
                if r_in > 0 {
                    return Some(f64_from_u64(r_out) / f64_from_u64(r_in));
                }
            }
            None
        });
    if let Some(retention) = read_retention {
        if retention < 0.4 {
            status = "fail".to_string();
            reasons.push(format!("read_retention {retention:.2} < 0.4"));
        } else if retention < 0.7 {
            status = "warn".to_string();
            reasons.push(format!("read_retention {retention:.2} < 0.7"));
        }
    }
    let mean_q = metrics.get("mean_q").and_then(serde_json::Value::as_f64);
    if let Some(mean_q) = mean_q {
        if mean_q < 15.0 {
            status = "fail".to_string();
            reasons.push(format!("mean_q {mean_q:.1} < 15"));
        } else if mean_q < 20.0 {
            status = "warn".to_string();
            reasons.push(format!("mean_q {mean_q:.1} < 20"));
        }
    }
    let mean_q_delta = metrics
        .get("delta_metrics")
        .and_then(|value| value.get("mean_q_delta"))
        .and_then(serde_json::Value::as_f64);
    if let Some(delta) = mean_q_delta {
        if delta < -1.0 {
            status = "warn".to_string();
            reasons.push(format!("mean_q_delta {delta:.2} < -1"));
        }
    }
    Some(serde_json::json!({
        "schema_version": "bijux.quality_gate.v1",
        "stage_id": stage_id,
        "status": status,
        "reasons": reasons,
        "thresholds": {
            "read_retention_warn": 0.7,
            "read_retention_fail": 0.4,
            "mean_q_warn": 20.0,
            "mean_q_fail": 15.0,
            "mean_q_delta_warn": -1.0
        }
    }))
}

#[derive(Debug, Default, Clone)]
#[allow(clippy::struct_field_names)]
struct FilterRemovalCounts {
    by_n: u64,
    by_entropy: u64,
    by_low_complexity: u64,
    by_kmer: u64,
    by_contaminant_kmer: u64,
    by_length: u64,
}

fn filter_removals_from_fastp(path: &Path) -> Option<FilterRemovalCounts> {
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let filtering = parsed.get("filtering_result")?;
    let by_n = filtering
        .get("too_many_N_reads")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let by_entropy = filtering
        .get("low_complexity_reads")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let by_length = filtering
        .get("too_short_reads")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        + filtering
            .get("too_long_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
    Some(FilterRemovalCounts {
        by_n,
        by_entropy,
        by_low_complexity: by_entropy,
        by_kmer: 0,
        by_contaminant_kmer: 0,
        by_length,
    })
}

fn filter_removals_from_bbduk_stats(
    path: &Path,
    kmer_ref_used: bool,
) -> Option<FilterRemovalCounts> {
    let raw = std::fs::read_to_string(path).ok()?;
    let mut removed = None;
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with("Reads Removed") || line.starts_with("Reads removed") {
            let digits: String = line.chars().filter(char::is_ascii_digit).collect();
            if !digits.is_empty() {
                removed = digits.parse::<u64>().ok();
            }
        }
    }
    let removed = removed?;
    Some(FilterRemovalCounts {
        by_n: 0,
        by_entropy: 0,
        by_low_complexity: 0,
        by_kmer: if kmer_ref_used { removed } else { 0 },
        by_contaminant_kmer: if kmer_ref_used { removed } else { 0 },
        by_length: 0,
    })
}

fn filter_removals_for_plan(
    tool_id: &str,
    out_dir: &Path,
    params: &serde_json::Value,
) -> FilterRemovalCounts {
    match tool_id {
        "fastp" => filter_removals_from_fastp(&out_dir.join("fastp.json")).unwrap_or_default(),
        "bbduk" => {
            let kmer_ref_used = params.get("kmer_ref").is_some();
            filter_removals_from_bbduk_stats(&out_dir.join("bbduk.stats"), kmer_ref_used)
                .unwrap_or_default()
        }
        _ => FilterRemovalCounts::default(),
    }
}

type IoDeltas = (
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
);

fn extract_io_deltas(metrics: &serde_json::Value) -> IoDeltas {
    let reads_in = metrics.get("reads_in").and_then(serde_json::Value::as_u64);
    let reads_out = metrics.get("reads_out").and_then(serde_json::Value::as_u64);
    let bases_in = metrics.get("bases_in").and_then(serde_json::Value::as_u64);
    let bases_out = metrics.get("bases_out").and_then(serde_json::Value::as_u64);
    let pairs_in = metrics.get("pairs_in").and_then(serde_json::Value::as_u64);
    let pairs_out = metrics.get("pairs_out").and_then(serde_json::Value::as_u64);
    (
        reads_in, reads_out, bases_in, bases_out, pairs_in, pairs_out,
    )
}

fn write_effective_fasta(
    run_artifacts_dir: &Path,
    name: &str,
    entries: &[BankEntryRecord],
    extra_fasta: &[String],
) -> Result<Option<(PathBuf, String)>> {
    if entries.is_empty() && extra_fasta.is_empty() {
        return Ok(None);
    }
    let banks_dir = run_artifacts_dir.join("banks");
    std::fs::create_dir_all(&banks_dir).context("create banks dir")?;
    let path = banks_dir.join(format!("effective_{name}.fasta"));
    let mut payload = String::new();
    for entry in entries {
        payload.push('>');
        payload.push_str(&entry.id);
        payload.push('\n');
        payload.push_str(&entry.sequence);
        payload.push('\n');
    }
    for fasta in extra_fasta {
        payload.push_str(fasta);
        if !fasta.ends_with('\n') {
            payload.push('\n');
        }
    }
    std::fs::write(&path, payload).context("write effective bank fasta")?;
    let hash = hash_file_sha256(&path)?;
    Ok(Some((path, hash)))
}

fn bank_asset_name(bank_name: &str) -> &str {
    match bank_name {
        "adapter" => "adapters",
        "contaminant" => "contaminants",
        other => other,
    }
}

fn write_effective_bank_yaml(
    run_artifacts_dir: &Path,
    name: &str,
    bank_value: &serde_json::Value,
    entries: &[BankEntryRecord],
    references: &[BankReferenceRecord],
) -> Result<Option<(PathBuf, String)>> {
    if entries.is_empty() && references.is_empty() {
        return Ok(None);
    }
    let banks_dir = run_artifacts_dir.join("banks");
    std::fs::create_dir_all(&banks_dir).context("create banks dir")?;
    let path = banks_dir.join(format!("effective_{name}.yaml"));
    let payload = serde_json::json!({
        "bank_id": bank_value.get("bank_id"),
        "bank_hash": bank_value.get("bank_hash"),
        "preset": bank_value.get("preset"),
        "preset_hash": bank_value.get("preset_hash"),
        "enabled_entries": entries.iter().map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "sequence": entry.sequence,
                "rationale": entry.rationale,
                "source": entry.source,
            })
        }).collect::<Vec<_>>(),
        "references": references.iter().map(|reference| {
            serde_json::json!({
                "id": reference.id,
                "file": reference.file,
                "sha256": reference.sha256,
                "rationale": reference.rationale,
                "source": reference.source,
            })
        }).collect::<Vec<_>>(),
    });
    let yaml = serde_yaml::to_string(&payload).context("serialize effective bank yaml")?;
    std::fs::write(&path, yaml).context("write effective bank yaml")?;
    let hash = hash_file_sha256(&path)?;
    Ok(Some((path, hash)))
}

fn write_effective_fasta_list(
    run_artifacts_dir: &Path,
    name: &str,
    references: &[BankReferenceRecord],
) -> Result<Option<(PathBuf, String)>> {
    if references.is_empty() {
        return Ok(None);
    }
    let banks_dir = run_artifacts_dir.join("banks");
    std::fs::create_dir_all(&banks_dir).context("create banks dir")?;
    let path = banks_dir.join(format!("effective_{name}.fasta.list"));
    let payload = references
        .iter()
        .map(|reference| reference.file.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&path, payload).context("write effective bank fasta list")?;
    let hash = hash_file_sha256(&path)?;
    Ok(Some((path, hash)))
}

fn materialize_bank_assets(
    run_artifacts_dir: &Path,
    banks_value: Option<&serde_json::Value>,
) -> Result<Option<serde_json::Value>> {
    let Some(banks_value) = banks_value.and_then(|value| value.as_object()) else {
        return Ok(None);
    };
    let mut assets = serde_json::Map::new();
    for (bank_name, bank_value) in banks_value {
        let asset_name = bank_asset_name(bank_name);
        let entries = bank_entries_from_value(bank_value);
        let references = bank_references_from_value(bank_value);
        let extra_fasta: Vec<String> = references
            .iter()
            .filter_map(|reference| reference.fasta.clone())
            .collect();
        let fasta = write_effective_fasta(run_artifacts_dir, asset_name, &entries, &extra_fasta)?;
        let yaml = write_effective_bank_yaml(
            run_artifacts_dir,
            asset_name,
            bank_value,
            &entries,
            &references,
        )?;
        let fasta_list = if bank_name.as_str() == "contaminant" {
            write_effective_fasta_list(run_artifacts_dir, asset_name, &references)?
        } else {
            None
        };
        let record = serde_json::json!({
            "yaml": yaml.as_ref().map(|(path, hash)| serde_json::json!({
                "path": path.display().to_string(),
                "sha256": hash,
            })),
            "fasta": fasta.as_ref().map(|(path, hash)| serde_json::json!({
                "path": path.display().to_string(),
                "sha256": hash,
            })),
            "fasta_list": fasta_list.as_ref().map(|(path, hash)| serde_json::json!({
                "path": path.display().to_string(),
                "sha256": hash,
            })),
        });
        assets.insert(bank_name.clone(), record);
    }
    Ok(Some(serde_json::Value::Object(assets)))
}

fn fastq_stats(path: &Path) -> Result<bijux_core::measure::SeqkitMetrics> {
    let file = std::fs::File::open(path).context("open fastq")?;
    let reader: Box<dyn std::io::Read> = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };
    let mut reads: u64 = 0;
    let mut bases: u64 = 0;
    let mut gc: u64 = 0;
    let mut q_sum: u64 = 0;
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next() {
        let header = line?;
        if header.is_empty() {
            continue;
        }
        let seq = lines
            .next()
            .ok_or_else(|| anyhow!("fastq missing sequence line"))??;
        let _plus = lines
            .next()
            .ok_or_else(|| anyhow!("fastq missing plus line"))??;
        let qual = lines
            .next()
            .ok_or_else(|| anyhow!("fastq missing quality line"))??;
        reads += 1;
        let seq_bytes = seq.as_bytes();
        bases += seq_bytes.len() as u64;
        for base in seq_bytes {
            match base {
                b'G' | b'g' | b'C' | b'c' => gc += 1,
                _ => {}
            }
        }
        for q in qual.as_bytes() {
            if *q >= 33 {
                q_sum += u64::from(q - 33);
            }
        }
    }
    let mean_q = if bases > 0 {
        f64_from_u64(q_sum) / f64_from_u64(bases)
    } else {
        0.0
    };
    let gc_percent = if bases > 0 {
        (f64_from_u64(gc) / f64_from_u64(bases)) * 100.0
    } else {
        0.0
    };
    Ok(bijux_core::measure::SeqkitMetrics {
        reads,
        bases,
        mean_q,
        gc_percent,
    })
}

fn pair_counts_from_paths(
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<(Option<u64>, Option<u64>)> {
    let pairs_in = if inputs.len() >= 2 {
        let r1 = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
        let r2 = stats_or_zero(inputs.get(1).map(PathBuf::as_path))?;
        Some(r1.reads.min(r2.reads))
    } else {
        None
    };
    let pairs_out = if outputs.len() >= 2 {
        let r1 = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
        let r2 = stats_or_zero(outputs.get(1).map(PathBuf::as_path))?;
        Some(r1.reads.min(r2.reads))
    } else {
        None
    };
    Ok((pairs_in, pairs_out))
}

fn stats_or_zero(path: Option<&Path>) -> Result<bijux_core::measure::SeqkitMetrics> {
    if let Some(path) = path {
        if path.exists() {
            if path.is_dir() {
                return Ok(bijux_core::measure::SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                });
            }
            if std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) == 0 {
                return Ok(bijux_core::measure::SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                });
            }
            return fastq_stats(path);
        }
    }
    Ok(bijux_core::measure::SeqkitMetrics {
        reads: 0,
        bases: 0,
        mean_q: 0.0,
        gc_percent: 0.0,
    })
}

fn stage_version_i32(version: bijux_core::StageVersion) -> i32 {
    i32::try_from(version.0).unwrap_or(i32::MAX)
}

fn observer_result_from_plan(
    plan: &StagePlanV1,
    outputs: Vec<PathBuf>,
    exit_code: i32,
    stdout: String,
    stderr: String,
) -> crate::core::types::StageResult {
    crate::core::types::StageResult {
        invocation: crate::core::types::ToolInvocation {
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            inputs: plan
                .io
                .inputs
                .iter()
                .map(|artifact| artifact.path.clone())
                .collect(),
            params: plan.params.clone(),
            requirements: None,
        },
        exit_code,
        stdout,
        stderr,
        outputs,
    }
}

#[derive(Debug, Clone, Copy)]
struct RetentionCounts {
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
}

#[allow(clippy::cast_precision_loss)]
fn f64_from_u64(value: u64) -> f64 {
    value as f64
}

#[allow(clippy::too_many_lines)]
fn stage_metrics_for_plan(
    stage_id: &str,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let metrics = match stage_id {
        "fastq.trim" => {
            let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let output = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let read_retention = if input.reads > 0 {
                f64_from_u64(output.reads) / f64_from_u64(input.reads)
            } else {
                0.0
            };
            let base_retention = if input.bases > 0 {
                f64_from_u64(output.bases) / f64_from_u64(input.bases)
            } else {
                0.0
            };
            let delta = FastqDeltaMetricsV1 {
                read_retention,
                base_retention,
                mean_q_delta: output.mean_q - input.mean_q,
                gc_delta: output.gc_percent - input.gc_percent,
            };
            let retention = RetentionReportMetricV1 {
                value: read_retention,
                numerator_reads: output.reads,
                denominator_reads: input.reads,
                numerator_bases: output.bases,
                denominator_bases: input.bases,
                definition: "reads_out / reads_in".to_string(),
                stage_boundary: stage_id.to_string(),
                conditions: retention_conditions_from_effective(stage_id, effective_params, params),
            };
            serde_json::to_value(FastqTrimMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
                mean_q_before: input.mean_q,
                mean_q_after: output.mean_q,
                delta_metrics: delta,
                retention,
            })?
        }
        "fastq.filter" => filter_metrics_with_removals(
            stage_id,
            inputs,
            outputs,
            params,
            effective_params,
            &FilterRemovalCounts::default(),
        )?,
        "fastq.merge" => {
            let r1 = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let r2 = stats_or_zero(inputs.get(1).map(PathBuf::as_path))?;
            let merged = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
            let unmerged_r1 = stats_or_zero(outputs.get(1).map(PathBuf::as_path))?;
            let unmerged_r2 = stats_or_zero(outputs.get(2).map(PathBuf::as_path))?;
            let reads_unmerged = unmerged_r1.reads.min(unmerged_r2.reads);
            let min_reads = r1.reads.min(r2.reads);
            let merge_rate = if min_reads > 0 {
                f64_from_u64(merged.reads) / f64_from_u64(min_reads)
            } else {
                0.0
            };
            let bases_in = r1.bases.min(r2.bases);
            serde_json::to_value(FastqMergeMetricsV1 {
                reads_in: min_reads,
                reads_out: merged.reads,
                bases_in,
                bases_out: merged.bases,
                pairs_in: min_reads,
                pairs_out: merged.reads,
                reads_r1: r1.reads,
                reads_r2: r2.reads,
                reads_merged: merged.reads,
                reads_unmerged,
                merge_rate,
            })?
        }
        "fastq.validate_pre" => {
            let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::to_value(FastqValidateMetricsV1 {
                reads_in: input.reads,
                reads_out: input.reads,
                bases_in: input.bases,
                bases_out: input.bases,
                pairs_in,
                pairs_out,
                reads_total: input.reads,
                reads_valid: input.reads,
                reads_invalid: 0,
                mean_q: input.mean_q,
            })?
        }
        "fastq.correct" => {
            let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let output = if outputs.is_empty() {
                input
            } else {
                stats_or_zero(outputs.first().map(PathBuf::as_path))?
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::to_value(FastqCorrectMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
            })?
        }
        "fastq.umi" => {
            let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let output = if outputs.is_empty() {
                input
            } else {
                stats_or_zero(outputs.first().map(PathBuf::as_path))?
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::to_value(FastqUmiMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
            })?
        }
        "fastq.preprocess" => {
            let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let output = if outputs.is_empty() {
                input
            } else {
                stats_or_zero(outputs.first().map(PathBuf::as_path))?
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::to_value(FastqPreprocessMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
            })?
        }
        "fastq.qc_post" => {
            let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let output = if outputs.is_empty() {
                input
            } else {
                stats_or_zero(outputs.first().map(PathBuf::as_path))?
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let out_dir = path_from_params(params, "out_dir")
                .unwrap_or_else(|| outputs.first().cloned().unwrap_or_default());
            let raw_dir = out_dir.join("fastqc_raw");
            let trimmed_dir = out_dir.join("fastqc_trimmed");
            let multiqc_report = out_dir.join("multiqc_report.html");
            let multiqc_data = out_dir.join("multiqc_data");
            serde_json::to_value(FastqQcPostMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
                mean_q: input.mean_q,
                contamination_rate: 0.0,
                raw_fastqc_dir: raw_dir.exists().then_some(raw_dir.display().to_string()),
                trimmed_fastqc_dir: trimmed_dir
                    .exists()
                    .then_some(trimmed_dir.display().to_string()),
                multiqc_report: multiqc_report
                    .exists()
                    .then_some(multiqc_report.display().to_string()),
                multiqc_data: multiqc_data
                    .exists()
                    .then_some(multiqc_data.display().to_string()),
            })?
        }
        "fastq.screen" => {
            let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let output = if outputs.is_empty() {
                input
            } else {
                stats_or_zero(outputs.first().map(PathBuf::as_path))?
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let report_path = path_from_params(params, "report")
                .or_else(|| outputs.first().cloned())
                .unwrap_or_else(|| PathBuf::from("screen_report.tsv"));
            let (contamination_rate, contamination_summary) = parse_screen_report(&report_path);
            serde_json::json!({
                "reads_in": input.reads,
                "reads_out": output.reads,
                "bases_in": input.bases,
                "bases_out": output.bases,
                "pairs_in": pairs_in,
                "pairs_out": pairs_out,
                "contamination_rate": contamination_rate,
                "contamination_summary": contamination_summary,
            })
        }
        "fastq.stats_neutral" => {
            let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let output = if outputs.is_empty() {
                input
            } else {
                stats_or_zero(outputs.first().map(PathBuf::as_path))?
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::json!({
                "reads_in": input.reads,
                "reads_out": output.reads,
                "bases_in": input.bases,
                "bases_out": output.bases,
                "pairs_in": pairs_in,
                "pairs_out": pairs_out,
            })
        }
        _ => serde_json::json!({}),
    };
    Ok(metrics)
}

fn filter_metrics_with_removals(
    stage_id: &str,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
    removals: &FilterRemovalCounts,
) -> Result<serde_json::Value> {
    let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
    let output = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
    let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
    let read_retention = if input.reads > 0 {
        f64_from_u64(output.reads) / f64_from_u64(input.reads)
    } else {
        0.0
    };
    let base_retention = if input.bases > 0 {
        f64_from_u64(output.bases) / f64_from_u64(input.bases)
    } else {
        0.0
    };
    let delta = FastqDeltaMetricsV1 {
        read_retention,
        base_retention,
        mean_q_delta: output.mean_q - input.mean_q,
        gc_delta: output.gc_percent - input.gc_percent,
    };
    let retention = RetentionReportMetricV1 {
        value: read_retention,
        numerator_reads: output.reads,
        denominator_reads: input.reads,
        numerator_bases: output.bases,
        denominator_bases: input.bases,
        definition: "reads_out / reads_in".to_string(),
        stage_boundary: stage_id.to_string(),
        conditions: retention_conditions_from_effective(stage_id, effective_params, params),
    };
    Ok(serde_json::to_value(FastqFilterMetricsV1 {
        reads_in: input.reads,
        reads_out: output.reads,
        reads_dropped: input.reads.saturating_sub(output.reads),
        reads_removed_by_n: removals.by_n,
        reads_removed_by_entropy: removals.by_entropy,
        reads_removed_low_complexity: removals.by_low_complexity,
        reads_removed_by_kmer: removals.by_kmer,
        reads_removed_contaminant_kmer: removals.by_contaminant_kmer,
        reads_removed_by_length: removals.by_length,
        bases_in: input.bases,
        bases_out: output.bases,
        pairs_in,
        pairs_out,
        mean_q_before: input.mean_q,
        mean_q_after: output.mean_q,
        delta_metrics: delta,
        retention,
    })?)
}

fn retention_counts_for_plan(
    stage_id: &str,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<Option<RetentionCounts>> {
    let counts = match stage_id {
        "fastq.trim" | "fastq.filter" | "fastq.correct" | "fastq.umi" | "fastq.preprocess" => {
            let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let output = if outputs.is_empty() {
                input
            } else {
                stats_or_zero(outputs.first().map(PathBuf::as_path))?
            };
            RetentionCounts {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
            }
        }
        "fastq.merge" => {
            let r1 = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let r2 = stats_or_zero(inputs.get(1).map(PathBuf::as_path))?;
            let merged = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
            RetentionCounts {
                reads_in: r1.reads.min(r2.reads),
                reads_out: merged.reads,
                bases_in: r1.bases.min(r2.bases),
                bases_out: merged.bases,
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(counts))
}

/// Execute a single stage plan.
///
/// # Errors
/// Returns an error if the execution fails or the plan is invalid.
#[allow(clippy::too_many_lines)]
pub fn execute_stage_plan(
    plan: &StagePlanV1,
    runner: RunnerKind,
    mut observer: Option<&mut dyn Observer>,
) -> Result<StageResultV1> {
    let run_id = Uuid::new_v4().to_string();
    let (r1, r2) = match plan.io.inputs.as_slice() {
        [] => (None, None),
        [r1] => (Some(r1.path.as_path()), None),
        [r1, r2, ..] => (Some(r1.path.as_path()), Some(r2.path.as_path())),
    };
    let r1 = r1.ok_or_else(|| anyhow!("plan inputs missing r1"))?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("input r1 has no parent directory"))?;
    let container_name = format!("bijux-stage-{}-{}", plan.stage_id.0, Uuid::new_v4());
    let run_artifacts_dir = run_artifacts_dir_for_out(&plan.out_dir);
    std::fs::create_dir_all(&run_artifacts_dir).context("create run_artifacts dir")?;
    let (trace_id, span_id) = default_trace_ids();
    let telemetry_path = std::env::var("BIJUX_TELEMETRY_JSONL").map_or_else(
        |_| run_artifacts_dir.join("telemetry").join("events.jsonl"),
        PathBuf::from,
    );
    let canonical_params = parameters_json_canonicalization(&plan.params);
    let sample_id = canonical_params
        .get("sample_id")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown")
        .to_string();
    let params_hash = params_hash(&canonical_params)?;
    let adapter_bank = adapter_bank_from_params(&canonical_params);
    let banks_json = banks_from_params(&canonical_params);
    let bank_assets = materialize_bank_assets(&run_artifacts_dir, banks_json.as_ref())?;
    let input_paths: Vec<PathBuf> = plan
        .io
        .inputs
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect();
    let input_hashes: Vec<String> = input_paths
        .iter()
        .map(|path| hash_file_sha256(path))
        .collect::<Result<Vec<_>>>()?;
    let input_hash = hash_inputs(&input_paths)?;
    let metric_context =
        metric_context_from_params(plan, runner, &input_hash, &params_hash, &canonical_params);
    let plan_artifacts = write_plan_artifacts(
        &run_artifacts_dir,
        &plan.stage_id.0,
        stage_version_i32(plan.stage_version),
        &plan.tool_id.0,
        &plan.tool_version,
        plan.image.digest.clone(),
        &runner.to_string(),
        &std::env::var("BIJUX_PLATFORM").unwrap_or_else(|_| "unknown".to_string()),
        &plan.resources,
        &plan
            .io
            .inputs
            .iter()
            .map(|artifact| artifact.path.clone())
            .collect::<Vec<_>>(),
        &plan
            .io
            .outputs
            .iter()
            .map(|artifact| artifact.path.clone())
            .collect::<Vec<_>>(),
        &canonical_params,
        &plan.effective_params,
        adapter_bank.as_ref(),
        banks_json.as_ref(),
        bank_assets.as_ref(),
    )?;
    let image = resolved_image_for_plan(&plan.image, runner);
    let image_digest = plan
        .image
        .digest
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let emit_event = |event: &bijux_core::TelemetryEventV1| -> Result<()> {
        write_telemetry_event(&telemetry_path, event)?;
        write_stage_event_jsonl(&run_artifacts_dir, event)?;
        Ok(())
    };
    let emit_artifact = |name: &str, path: &Path| -> Result<()> {
        emit_event(&bijux_core::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.clone(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            event_name: "artifact_written".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            attrs: serde_json::json!({
                "artifact": name,
                "path": path.display().to_string(),
            }),
        })
    };
    emit_event(&bijux_core::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: plan.stage_id.0.clone(),
        tool_id: plan.tool_id.0.clone(),
        event_name: "stage_start".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        duration_ms: None,
        status: "ok".to_string(),
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        attrs: serde_json::json!({
            "params_hash": &params_hash,
            "input_hash": &input_hash,
            "runner": format!("{:?}", runner),
            "image": image.full_name.clone(),
            "image_digest": image_digest,
            "tool_version": plan.tool_version.clone(),
        }),
    })?;
    emit_event(&bijux_core::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: plan.stage_id.0.clone(),
        tool_id: plan.tool_id.0.clone(),
        event_name: "tool_start".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        duration_ms: None,
        status: "ok".to_string(),
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        attrs: serde_json::json!({
            "params_hash": &params_hash,
            "input_hash": &input_hash,
            "runner": format!("{:?}", runner),
            "image": image.full_name.clone(),
            "image_digest": image_digest,
            "tool_version": plan.tool_version.clone(),
        }),
    })?;
    let started_at = Utc::now();
    let start = Instant::now();
    let mut outputs_override: Option<Vec<PathBuf>> = None;
    let mut telemetry_exit_code: Option<i32> = None;
    let mut telemetry_output_hashes: Vec<String> = Vec::new();
    let mut telemetry_error: Option<String> = None;
    if let Some(observer) = observer.as_mut() {
        let start_result =
            observer_result_from_plan(plan, Vec::new(), -1, String::new(), String::new());
        observer.on_stage_start(&start_result)?;
    }
    info!(
        run_id = %run_id,
        sample_id = %sample_id,
        stage = %plan.stage_id.0,
        tool = %plan.tool_id.0,
        tool_version = %plan.tool_version,
        image_digest = %plan.image.digest.clone().unwrap_or_else(|| "unknown".to_string()),
        params_hash = %params_hash,
        input_hash = %input_hash,
        "stage execution starting"
    );
    let result: Result<StageResultV1> = (|| {
        let execution = match plan.stage_id.0.as_str() {
            "fastq.merge" => {
                let r2 = r2.ok_or_else(|| anyhow!("merge requires r2 input"))?;
                let exec = run_merge_execution(
                    &plan.tool_id.0,
                    &image,
                    r1_dir,
                    r1,
                    r2,
                    &plan.out_dir,
                    &container_name,
                )?;
                outputs_override = Some(vec![
                    exec.merged_fastq.clone(),
                    exec.unmerged_r1.clone(),
                    exec.unmerged_r2.clone(),
                ]);
                ExecutionEnvelope {
                    exit_code: exec.exit_code,
                    stdout: exec.stdout,
                    stderr: exec.stderr,
                    command: exec.command,
                }
            }
            "fastq.qc_post" if plan.tool_id.0 == "multiqc" => {
                let fastqc_image = plan
                    .aux_images
                    .get("fastqc")
                    .ok_or_else(|| anyhow!("fastqc image missing for multiqc qc_post"))?;
                let fastqc_image = resolved_image_for_plan(fastqc_image, runner);
                let fastqc_trimmed_dir = plan.out_dir.join("fastqc_trimmed");
                std::fs::create_dir_all(&fastqc_trimmed_dir)?;
                let fastqc_trimmed_container = format!("bijux-stage-fastqc-{}", Uuid::new_v4());
                let fastqc_trimmed_exec = run_validate_execution(
                    "fastqc",
                    &fastqc_image,
                    r1_dir,
                    r1,
                    &fastqc_trimmed_dir,
                    &fastqc_trimmed_container,
                )?;
                cleanup_execution(&fastqc_trimmed_container)?;
                if fastqc_trimmed_exec.exit_code != 0 {
                    return Err(anyhow!(
                        "fastqc trimmed exit code {}",
                        fastqc_trimmed_exec.exit_code
                    ));
                }

                if let Some(raw_r1) = canonical_params
                    .get("raw_r1")
                    .and_then(|value| value.as_str())
                {
                    let raw_r1 = PathBuf::from(raw_r1);
                    if let Some(raw_dir) = raw_r1.parent() {
                        let fastqc_raw_dir = plan.out_dir.join("fastqc_raw");
                        std::fs::create_dir_all(&fastqc_raw_dir)?;
                        let fastqc_raw_container = format!("bijux-stage-fastqc-{}", Uuid::new_v4());
                        let fastqc_raw_exec = run_validate_execution(
                            "fastqc",
                            &fastqc_image,
                            raw_dir,
                            &raw_r1,
                            &fastqc_raw_dir,
                            &fastqc_raw_container,
                        )?;
                        cleanup_execution(&fastqc_raw_container)?;
                        if fastqc_raw_exec.exit_code != 0 {
                            return Err(anyhow!(
                                "fastqc raw exit code {}",
                                fastqc_raw_exec.exit_code
                            ));
                        }
                    }
                }

                let exec =
                    run_multiqc_execution(&image, &plan.out_dir, &plan.out_dir, &container_name)?;
                ExecutionEnvelope {
                    exit_code: exec.exit_code,
                    stdout: exec.stdout,
                    stderr: exec.stderr,
                    command: exec.command,
                }
            }
            "fastq.validate_pre" | "fastq.qc_post" => {
                let exec = run_validate_execution(
                    &plan.tool_id.0,
                    &image,
                    r1_dir,
                    r1,
                    &plan.out_dir,
                    &container_name,
                )?;
                ExecutionEnvelope {
                    exit_code: exec.exit_code,
                    stdout: exec.stdout,
                    stderr: exec.stderr,
                    command: exec.command,
                }
            }
            "fastq.filter" => {
                let mut filter_params = canonical_params.clone();
                if let Some(kmer_ref) = canonical_params
                    .get("kmer_ref")
                    .and_then(|value| value.as_str())
                {
                    let src = PathBuf::from(kmer_ref);
                    if src.exists() {
                        let dest = plan.out_dir.join("kmer_ref.fasta");
                        std::fs::copy(&src, &dest)?;
                        if let Some(obj) = filter_params.as_object_mut() {
                            obj.insert(
                                "kmer_ref".to_string(),
                                serde_json::Value::String(
                                    "/data/output/kmer_ref.fasta".to_string(),
                                ),
                            );
                        }
                    }
                }
                let exec = run_filter_execution(
                    &plan.tool_id.0,
                    &image,
                    r1_dir,
                    r1,
                    &plan.out_dir,
                    &container_name,
                    &filter_params,
                )?;
                ExecutionEnvelope {
                    exit_code: exec.exit_code,
                    stdout: exec.stdout,
                    stderr: exec.stderr,
                    command: exec.command,
                }
            }
            _ => {
                let exec = run_tool_execution(
                    &plan.tool_id.0,
                    &image,
                    r1_dir,
                    r1,
                    &plan.out_dir,
                    &container_name,
                )?;
                ExecutionEnvelope {
                    exit_code: exec.exit_code,
                    stdout: exec.stdout,
                    stderr: exec.stderr,
                    command: exec.command,
                }
            }
        };
        telemetry_exit_code = Some(execution.exit_code);
        let runtime_s = start.elapsed().as_secs_f64();
        let memory_mb = execution_memory_mb(&container_name)?;
        cleanup_execution(&container_name)?;
        let outputs = outputs_override.unwrap_or_else(|| {
            plan.io
                .outputs
                .iter()
                .map(|artifact| artifact.path.clone())
                .collect()
        });
        let output_hashes = hash_outputs(&outputs)?;
        telemetry_output_hashes.clone_from(&output_hashes);
        let stage_metrics = if plan.stage_id.0 == "fastq.filter" {
            let removals =
                filter_removals_for_plan(plan.tool_id.0.as_str(), &plan.out_dir, &canonical_params);
            filter_metrics_with_removals(
                plan.stage_id.0.as_str(),
                &input_paths,
                &outputs,
                &canonical_params,
                &plan.effective_params,
                &removals,
            )?
        } else {
            stage_metrics_for_plan(
                plan.stage_id.0.as_str(),
                &input_paths,
                &outputs,
                &canonical_params,
                &plan.effective_params,
            )?
        };
        let invocation = ToolInvocationV1 {
            schema_version: "bijux.tool_invocation.v1".to_string(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            tool_version: plan.tool_version.clone(),
            resolved_tool_version: Some(plan.tool_version.clone()),
            image_digest: image_digest.clone(),
            runner_kind: runner.to_string(),
            platform: std::env::var("BIJUX_PLATFORM").unwrap_or_else(|_| "unknown".to_string()),
            parameters_json: canonical_params.clone(),
            parameters_json_normalized: bijux_core::parameters_json_canonicalization(
                &canonical_params,
            ),
            effective_params_json: plan.effective_params.clone(),
            effective_params_json_normalized: bijux_core::parameters_json_canonicalization(
                &plan.effective_params,
            ),
            adapter_bank: adapter_bank_from_params(&canonical_params),
            banks: banks_json.clone(),
            bank_assets: bank_assets.clone(),
            resources: plan.resources.clone(),
            environment: std::env::vars().collect::<BTreeMap<String, String>>(),
            input_hashes: input_hashes.clone(),
            output_hashes: output_hashes.clone(),
            executed_command: Some(execution.command.clone()),
        };
        let tool_invocation_path =
            write_tool_invocation_json(&run_artifacts_dir, &plan.stage_id.0, &invocation)?;
        emit_artifact("tool_invocation", &tool_invocation_path)?;
        let ctx = StageObservabilityContextV1 {
            stage_id: plan.stage_id.0.clone(),
            stage_version: stage_version_i32(plan.stage_version),
            tool_id: plan.tool_id.0.clone(),
            tool_version: plan.tool_version.clone(),
            input_hash: input_hash.clone(),
            params_hash: params_hash.clone(),
            parameters_json: canonical_params.clone(),
            metric_context: metric_context.clone(),
        };
        let execution_metrics = bijux_core::measure::ExecutionMetrics {
            runtime_s,
            memory_mb,
            exit_code: execution.exit_code,
        };
        let metrics_envelope_path = write_metrics_envelope(
            &run_artifacts_dir,
            &ctx,
            &execution_metrics,
            &stage_metrics,
            &output_hashes,
        )?;
        emit_artifact("metrics_envelope", &metrics_envelope_path)?;
        let stage_metrics_payload = StageMetricsV1 {
            schema_version: "bijux.stage_metrics.v1".to_string(),
            stage_id: plan.stage_id.0.clone(),
            stage_version: stage_version_i32(plan.stage_version),
            tool_id: plan.tool_id.0.clone(),
            tool_version: plan.tool_version.clone(),
            context: metric_context.clone(),
            execution: execution_metrics,
            failure_class: None,
            failure_reason: None,
            metrics: stage_metrics.clone(),
        };
        let stage_metrics_path =
            write_stage_metrics_json(&run_artifacts_dir, &stage_metrics_payload)?;
        emit_artifact("stage_metrics", &stage_metrics_path)?;
        let metrics_path = run_artifacts_dir.join("metrics.json");
        let facts_row_id = format!("{}:{}:{}", run_id, plan.stage_id.0, plan.tool_id.0);
        let mut subreports: Vec<PathBuf> = Vec::new();
        let mut extra_warnings: Vec<String> = Vec::new();
        let mut adapter_validation: Option<serde_json::Value> = None;
        let mut contaminant_action = false;
        if let Some(banks_value) = banks_json.as_ref().and_then(|value| value.as_object()) {
            let mut banks_report = serde_json::Map::new();
            for (bank_name, bank_value) in banks_value {
                let entries = bank_entries_from_value(bank_value);
                let references = bank_references_from_value(bank_value);
                let assets_for_bank = bank_assets
                    .as_ref()
                    .and_then(|assets| assets.get(bank_name))
                    .cloned();
                let bank_entry_report = serde_json::json!({
                    "bank_id": bank_value.get("bank_id"),
                    "bank_hash": bank_value.get("bank_hash"),
                    "preset": bank_value.get("preset"),
                    "preset_hash": bank_value.get("preset_hash"),
                    "enabled_entries": entries.iter().map(|entry| {
                        serde_json::json!({
                            "id": entry.id,
                            "rationale": entry.rationale,
                            "source": entry.source,
                        })
                    }).collect::<Vec<_>>(),
                    "references": references.iter().map(|reference| {
                        serde_json::json!({
                            "id": reference.id,
                            "file": reference.file,
                            "sha256": reference.sha256,
                            "rationale": reference.rationale,
                            "source": reference.source,
                        })
                    }).collect::<Vec<_>>(),
                    "assets": assets_for_bank,
                });
                banks_report.insert(bank_name.clone(), bank_entry_report);
            }
            let report_payload = serde_json::json!({
                "schema_version": "bijux.bank_report.v1",
                "stage_id": plan.stage_id.0,
                "tool_id": plan.tool_id.0,
                "banks": banks_report,
            });
            let reports_dir = run_artifacts_dir.join("reports");
            std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
            let report_path = reports_dir.join("bank_report.json");
            std::fs::write(&report_path, serde_json::to_vec_pretty(&report_payload)?)
                .context("write bank_report.json")?;
            emit_artifact("bank_report", &report_path)?;
            subreports.push(report_path);
        }
        if plan.stage_id.0 == "fastq.trim" {
            let input = stats_or_zero(input_paths.first().map(PathBuf::as_path))?;
            let output = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
            let adapter_bank = canonical_params.get("adapter_bank");
            let adapter_preset = adapter_bank
                .and_then(|value| value.get("preset"))
                .and_then(|value| value.as_str())
                .map(str::to_string);
            let adapter_bank_id = adapter_bank
                .and_then(|value| value.get("bank_id"))
                .and_then(|value| value.as_str())
                .map(str::to_string);
            let adapter_bank_hash = adapter_bank
                .and_then(|value| value.get("bank_hash"))
                .and_then(|value| value.as_str())
                .map(str::to_string);
            let adapter_overrides = canonical_params.get("adapter_overrides").cloned();
            let report_path = write_trim_report_v1(
                &run_artifacts_dir,
                &plan.stage_id.0,
                &plan.tool_id.0,
                input.reads,
                output.reads,
                input.bases,
                output.bases,
                adapter_preset,
                adapter_bank_id,
                adapter_bank_hash,
                adapter_overrides,
            )?;
            emit_artifact("trim_report", &report_path)?;
            subreports.push(report_path);
        }
        if plan.stage_id.0 == "fastq.validate_pre" {
            let input = stats_or_zero(input_paths.first().map(PathBuf::as_path))?;
            let report_path = write_validate_report_v1(
                &run_artifacts_dir,
                &plan.stage_id.0,
                &plan.tool_id.0,
                input.reads,
                input.reads,
                0,
            )?;
            emit_artifact("validate_report", &report_path)?;
            subreports.push(report_path);
        }
        if plan.stage_id.0 == "fastq.filter" {
            let input = stats_or_zero(input_paths.first().map(PathBuf::as_path))?;
            let output = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
            let redundant_filters = canonical_params
                .get("redundant_filters")
                .and_then(|value| value.as_array())
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(str::to_string))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();
            let removals =
                filter_removals_for_plan(plan.tool_id.0.as_str(), &plan.out_dir, &canonical_params);
            let report_path = write_filter_report_v1(
                &run_artifacts_dir,
                &plan.stage_id.0,
                &plan.tool_id.0,
                input.reads,
                output.reads,
                input.reads.saturating_sub(output.reads),
                removals.by_n,
                removals.by_entropy,
                removals.by_low_complexity,
                removals.by_kmer,
                removals.by_contaminant_kmer,
                removals.by_length,
                serde_json::json!({}),
                retention_conditions_from_effective(
                    &plan.stage_id.0,
                    &plan.effective_params,
                    &canonical_params,
                ),
                redundant_filters,
            )?;
            emit_artifact("filter_report", &report_path)?;
            subreports.push(report_path);
        }
        if plan.stage_id.0 == "fastq.merge" {
            let r1 = stats_or_zero(input_paths.first().map(PathBuf::as_path))?;
            let r2 = stats_or_zero(input_paths.get(1).map(PathBuf::as_path))?;
            let merged = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
            let unmerged_r1 = stats_or_zero(outputs.get(1).map(PathBuf::as_path))?;
            let unmerged_r2 = stats_or_zero(outputs.get(2).map(PathBuf::as_path))?;
            let reads_unmerged = unmerged_r1.reads.min(unmerged_r2.reads);
            let min_reads = r1.reads.min(r2.reads);
            let merge_rate = if min_reads > 0 {
                f64_from_u64(merged.reads) / f64_from_u64(min_reads)
            } else {
                0.0
            };
            let report_path = write_merge_report_v1(
                &run_artifacts_dir,
                &plan.stage_id.0,
                &plan.tool_id.0,
                r1.reads,
                r2.reads,
                merged.reads,
                reads_unmerged,
                merge_rate,
            )?;
            emit_artifact("merge_report", &report_path)?;
            subreports.push(report_path);
        }
        if plan.stage_id.0 == "fastq.qc_post" {
            let raw_dir = plan.out_dir.join("fastqc_raw");
            let trimmed_dir = plan.out_dir.join("fastqc_trimmed");
            let raw_modules = fastqc_modules_from_dir(&raw_dir);
            let trimmed_modules = fastqc_modules_from_dir(&trimmed_dir);
            let raw_dir_opt = if raw_dir.exists() {
                Some(raw_dir.as_path())
            } else {
                None
            };
            let trimmed_dir_opt = if trimmed_dir.exists() {
                Some(trimmed_dir.as_path())
            } else {
                None
            };
            let multiqc_report = plan.out_dir.join("multiqc_report.html");
            let multiqc_data = plan.out_dir.join("multiqc_data");
            let (suggested_payload, suggested_preset) = if raw_dir.exists() {
                adapter_suggestions_from_fastqc(&raw_dir)
            } else {
                adapter_suggestions_from_fastqc(&trimmed_dir)
            };
            if let Some(preset) = suggested_preset.as_deref() {
                let current = canonical_params
                    .get("adapter_bank")
                    .and_then(|value| value.get("preset"))
                    .and_then(|value| value.as_str());
                if current == Some("illumina-default") {
                    adapter_validation = Some(serde_json::json!({
                        "current_preset": current,
                        "suggested_preset": preset,
                        "status": "warn",
                    }));
                    extra_warnings.push(format!(
                        "warning: adapter signal detected; consider preset '{preset}'"
                    ));
                    emit_event(&bijux_core::TelemetryEventV1 {
                        schema_version: "bijux.telemetry.v1".to_string(),
                        run_id: run_id.clone(),
                        stage_id: plan.stage_id.0.clone(),
                        tool_id: plan.tool_id.0.clone(),
                        event_name: "adapter_validation".to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        duration_ms: None,
                        status: "warn".to_string(),
                        trace_id: trace_id.clone(),
                        span_id: span_id.clone(),
                        attrs: serde_json::json!({
                            "current_preset": current,
                            "suggested_preset": preset,
                        }),
                    })?;
                }
            }
            let suggested_path = if suggested_payload
                .as_object()
                .is_none_or(serde_json::Map::is_empty)
            {
                None
            } else {
                let reports_dir = run_artifacts_dir.join("reports");
                std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
                let path = reports_dir.join("suggested_adapters.json");
                std::fs::write(&path, serde_json::to_vec_pretty(&suggested_payload)?)
                    .context("write suggested adapters")?;
                emit_artifact("suggested_adapters", &path)?;
                Some(path)
            };

            let report_path = write_qc_post_report_v1(
                &run_artifacts_dir,
                &plan.stage_id.0,
                &plan.tool_id.0,
                raw_dir_opt,
                trimmed_dir_opt,
                multiqc_report.exists().then_some(multiqc_report.as_path()),
                multiqc_data.exists().then_some(multiqc_data.as_path()),
                raw_modules,
                trimmed_modules,
                suggested_path.as_deref(),
                suggested_preset.as_deref(),
            )?;
            emit_artifact("qc_post_report", &report_path)?;
            subreports.push(report_path);
        }
        let mut warnings = warnings_for_plan(plan, &canonical_params);
        warnings.extend(extra_warnings);
        let (reads_in, reads_out, bases_in, bases_out, pairs_in, pairs_out) =
            extract_io_deltas(&stage_metrics);
        let thresholds = thresholds_from_env();
        let invariant_eval = evaluate_invariants(
            &plan.stage_id.0,
            &stage_metrics,
            &plan.effective_params,
            &thresholds,
        );
        let assertion_results = invariant_eval.results.clone();
        let assertions_payload = serde_json::json!({
            "schema_version": "bijux.assertions.v1",
            "results": assertion_results,
        });
        let scientific_preset = std::env::var("BIJUX_SCIENTIFIC_PRESET").ok();
        if plan.stage_id.0 == "fastq.filter" && canonical_params.get("kmer_ref").is_some() {
            contaminant_action = true;
            emit_event(&bijux_core::TelemetryEventV1 {
                schema_version: "bijux.telemetry.v1".to_string(),
                run_id: run_id.clone(),
                stage_id: plan.stage_id.0.clone(),
                tool_id: plan.tool_id.0.clone(),
                event_name: "contaminant_action".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                duration_ms: None,
                status: "ok".to_string(),
                trace_id: trace_id.clone(),
                span_id: span_id.clone(),
                attrs: serde_json::json!({
                    "enabled": true,
                    "kmer_ref": canonical_params.get("kmer_ref"),
                }),
            })?;
        }
        let quality_gate =
            quality_gate_decision(&plan.stage_id.0, &stage_metrics, reads_in, reads_out);
        if let Some(decision) = quality_gate.as_ref() {
            let status = decision
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or("pass");
            emit_event(&bijux_core::TelemetryEventV1 {
                schema_version: "bijux.telemetry.v1".to_string(),
                run_id: run_id.clone(),
                stage_id: plan.stage_id.0.clone(),
                tool_id: plan.tool_id.0.clone(),
                event_name: "quality_gate".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                duration_ms: None,
                status: status.to_string(),
                trace_id: trace_id.clone(),
                span_id: span_id.clone(),
                attrs: decision.clone(),
            })?;
            if status != "pass" {
                warnings.push(format!(
                    "warning: quality gate {} for stage {}",
                    status, plan.stage_id.0
                ));
            }
        }
        let stage_report_path = write_stage_report_v1(
            &run_artifacts_dir,
            &plan.stage_id.0,
            stage_version_i32(plan.stage_version),
            &plan.tool_id.0,
            &plan.tool_version,
            &metrics_path,
            &tool_invocation_path,
            &plan_artifacts.effective_config_path,
            Some(&facts_row_id),
            &outputs,
            &subreports,
            &[],
            &warnings,
            &[],
            &assertion_results,
            Some(&invariant_eval.verdict),
        )?;
        emit_artifact("stage_report", &stage_report_path)?;
        for warning in &warnings {
            emit_event(&bijux_core::TelemetryEventV1 {
                schema_version: "bijux.telemetry.v1".to_string(),
                run_id: run_id.clone(),
                stage_id: plan.stage_id.0.clone(),
                tool_id: plan.tool_id.0.clone(),
                event_name: "warn".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                duration_ms: None,
                status: "warn".to_string(),
                trace_id: trace_id.clone(),
                span_id: span_id.clone(),
                attrs: serde_json::json!({
                    "message": warning,
                }),
            })?;
        }
        let retention_report_path = if is_retention_stage(&plan.stage_id.0) {
            retention_counts_for_plan(&plan.stage_id.0, &input_paths, &outputs)?.map(|counts| {
                write_retention_report_v1(
                    &run_artifacts_dir,
                    &plan.stage_id.0,
                    &plan.tool_id.0,
                    &plan.tool_version,
                    &retention_conditions_from_effective(
                        &plan.stage_id.0,
                        &plan.effective_params,
                        &canonical_params,
                    ),
                    &canonical_params,
                    counts.reads_in,
                    counts.reads_out,
                    counts.bases_in,
                    counts.bases_out,
                )
            })
        } else {
            None
        }
        .transpose()?;
        if let Some(retention_path) = retention_report_path.as_ref() {
            emit_artifact("retention_report", retention_path)?;
        }
        let effective_adapters_path = match adapter_bank.as_ref() {
            Some(bank) => write_effective_adapters_from_provenance(&run_artifacts_dir, bank)?,
            None => None,
        };
        if let Some(path) = effective_adapters_path.as_ref() {
            emit_artifact("effective_adapters", path)?;
        }
        let mut extra_manifest_artifacts = Vec::new();
        if let Some(path) = effective_adapters_path.as_ref() {
            extra_manifest_artifacts.push(serde_json::json!({
                "name": "effective_adapters",
                "path": path,
            }));
        }
        let _observability_manifest = write_observability_manifest(
            &run_artifacts_dir,
            &plan.stage_id.0,
            &plan.tool_id.0,
            &plan_artifacts.plan_path,
            &plan_artifacts.effective_config_path,
            &plan_artifacts.stage_config_path,
            &tool_invocation_path,
            &metrics_envelope_path,
            &stage_metrics_path,
            &stage_report_path,
            retention_report_path.as_deref(),
            &extra_manifest_artifacts,
        )?;
        let _ = insert_stage_row(
            &run_artifacts_dir.join("run_index.jsonl"),
            &StageIndexRow {
                run_id: run_id.clone(),
                stage_id: plan.stage_id.0.clone(),
                tool_id: plan.tool_id.0.clone(),
                params_hash: params_hash.clone(),
                input_hash: input_hash.clone(),
                output_hashes: output_hashes.clone(),
                artifacts: serde_json::json!({
                    "plan": plan_artifacts.plan_path.display().to_string(),
                    "effective_config": plan_artifacts.effective_config_path.display().to_string(),
                    "stage_config": plan_artifacts.stage_config_path.display().to_string(),
                    "metrics_envelope": metrics_envelope_path.display().to_string(),
                    "stage_report": stage_report_path.display().to_string(),
                    "retention_report": retention_report_path.as_ref().map(|path| path.display().to_string()),
                    "effective_adapters": effective_adapters_path.as_ref().map(|path| path.display().to_string()),
                }),
            },
        );
        write_facts_jsonl(
            &run_artifacts_dir.join("dashboard").join("facts.jsonl"),
            &FactsRowV1 {
                schema_version: "bijux.facts.v1".to_string(),
                run_id: run_id.clone(),
                stage_id: plan.stage_id.0.clone(),
                tool_id: plan.tool_id.0.clone(),
                tool_version: plan.tool_version.clone(),
                image_digest: plan.image.digest.clone(),
                trace_id: trace_id.clone(),
                span_id: span_id.clone(),
                params_hash: params_hash.clone(),
                input_hash: input_hash.clone(),
                output_hashes: output_hashes.clone(),
                runtime_s,
                memory_mb,
                exit_code: execution.exit_code,
                bank_hashes: bank_refs_from_params(&canonical_params),
                reads_in,
                reads_out,
                bases_in,
                bases_out,
                pairs_in,
                pairs_out,
                metrics: stage_metrics.clone(),
                reports: serde_json::json!({
                    "stage_report": stage_report_path.display().to_string(),
                    "retention_report": retention_report_path.as_ref().map(|path| path.display().to_string()),
                    "bank_report": subreports.iter().find(|path| path.ends_with("bank_report.json")).map(|path| path.display().to_string()),
                    "qc_post_report": subreports.iter().find(|path| path.ends_with("qc_post_report.json")).map(|path| path.display().to_string()),
                    "filter_report": subreports.iter().find(|path| path.ends_with("filter_report.json")).map(|path| path.display().to_string()),
                    "quality_gate": quality_gate,
                    "adapter_validation": adapter_validation,
                    "contaminant_action": contaminant_action,
                    "assertions": assertions_payload,
                    "scientific_preset": scientific_preset,
                }),
                artifacts: serde_json::json!({
                    "metrics_envelope": metrics_envelope_path.display().to_string(),
                    "stage_report": stage_report_path.display().to_string(),
                    "retention_report": retention_report_path.as_ref().map(|path| path.display().to_string()),
                    "effective_adapters": effective_adapters_path.as_ref().map(|path| path.display().to_string()),
                }),
            },
        )?;
        let finished_at = Utc::now();
        let progress_status = if execution.exit_code == 0 {
            "ok"
        } else {
            "error"
        };
        write_progress_event_jsonl(
            &run_artifacts_dir,
            &crate::services::run_artifacts::ProgressEventV1 {
                schema_version: "bijux.progress.v1",
                stage_id: plan.stage_id.0.clone(),
                tool_id: plan.tool_id.0.clone(),
                status: progress_status.to_string(),
                started_at: started_at.to_rfc3339(),
                finished_at: finished_at.to_rfc3339(),
                outputs: outputs
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect(),
                metrics_path: Some(metrics_envelope_path.display().to_string()),
            },
        )?;
        write_runs_export_jsonl(
            &run_artifacts_dir,
            &crate::services::run_artifacts::RunsExportRowV1 {
                schema_version: "bijux.runs_export.v1",
                run_id: run_id.clone(),
                stage_id: plan.stage_id.0.clone(),
                tool_id: plan.tool_id.0.clone(),
                tool_version: plan.tool_version.clone(),
                started_at: started_at.to_rfc3339(),
                finished_at: finished_at.to_rfc3339(),
                runtime_s,
                memory_mb,
                exit_code: execution.exit_code,
                params_hash: params_hash.clone(),
                input_hash: input_hash.clone(),
                metrics_path: Some(metrics_envelope_path.display().to_string()),
            },
        )?;
        let marker_path = plan.out_dir.join("engine_execution.json");
        let marker = serde_json::json!({
            "schema_version": "bijux.engine_execution.v1",
            "stage": plan.stage_id.0,
            "tool": plan.tool_id.0,
        });
        std::fs::write(&marker_path, serde_json::to_vec_pretty(&marker)?)
            .context("write engine execution marker")?;
        let stage_result = StageResultV1 {
            run_id: run_id.clone(),
            exit_code: execution.exit_code,
            runtime_s,
            memory_mb,
            outputs,
            metrics_path: Some(metrics_envelope_path),
            stdout: execution.stdout,
            stderr: execution.stderr,
            command: execution.command,
        };
        info!(
            run_id = %run_id,
            sample_id = %sample_id,
            stage = %plan.stage_id.0,
            tool = %plan.tool_id.0,
            tool_version = %plan.tool_version,
            image_digest = %image_digest,
            params_hash = %params_hash,
            input_hash = %input_hash,
            exit_code = execution.exit_code,
            runtime_s = runtime_s,
            memory_mb = memory_mb,
            "stage execution finished"
        );
        if let Some(observer) = observer.as_mut() {
            let observer_result = observer_result_from_plan(
                plan,
                stage_result.outputs.clone(),
                stage_result.exit_code,
                stage_result.stdout.clone(),
                stage_result.stderr.clone(),
            );
            observer.on_stage_end(&observer_result)?;
        }
        Ok(stage_result)
    })();
    let runtime_s = start.elapsed().as_secs_f64();
    let duration_ms = {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            (runtime_s * 1000.0).max(0.0) as u64
        }
    };
    if let Err(err) = &result {
        let _ = cleanup_execution(&container_name);
        telemetry_error = Some(err.to_string());
    }
    let status = match telemetry_exit_code {
        Some(0) if result.is_ok() => "ok",
        _ => "error",
    };
    let exit_code = telemetry_exit_code.unwrap_or(-1);
    emit_event(&bijux_core::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: plan.stage_id.0.clone(),
        tool_id: plan.tool_id.0.clone(),
        event_name: "tool_end".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        duration_ms: Some(duration_ms),
        status: status.to_string(),
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        attrs: serde_json::json!({
            "exit_code": exit_code,
            "params_hash": &params_hash,
            "input_hash": &input_hash,
            "output_hashes": &telemetry_output_hashes,
            "runner": format!("{:?}", runner),
            "image": image.full_name.clone(),
            "image_digest": image_digest,
            "error": telemetry_error.clone(),
        }),
    })?;
    emit_event(&bijux_core::TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: run_id.clone(),
        stage_id: plan.stage_id.0.clone(),
        tool_id: plan.tool_id.0.clone(),
        event_name: "stage_end".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        duration_ms: Some(duration_ms),
        status: status.to_string(),
        trace_id: trace_id.clone(),
        span_id: span_id.clone(),
        attrs: serde_json::json!({
            "exit_code": exit_code,
            "params_hash": &params_hash,
            "input_hash": &input_hash,
            "output_hashes": &telemetry_output_hashes,
            "runner": format!("{:?}", runner),
            "image": image.full_name.clone(),
            "image_digest": image_digest,
            "error": telemetry_error.clone(),
        }),
    })?;
    if let Some(error) = telemetry_error.as_ref() {
        emit_event(&bijux_core::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.clone(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            event_name: "error".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "error".to_string(),
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            attrs: serde_json::json!({
                "message": error,
                "exit_code": exit_code,
            }),
        })?;
    }
    result
}

fn hash_inputs(inputs: &[PathBuf]) -> Result<String> {
    if inputs.is_empty() {
        return Ok("none".to_string());
    }
    let mut hashes = Vec::new();
    for input in inputs {
        hashes.push(hash_file_sha256(input)?);
    }
    Ok(hashes.join(","))
}

fn hash_outputs(outputs: &[PathBuf]) -> Result<Vec<String>> {
    let mut hashes = Vec::new();
    for output in outputs {
        if output.is_file() {
            hashes.push(hash_file_sha256(output)?);
        }
    }
    Ok(hashes)
}

fn is_retention_stage(stage_id: &str) -> bool {
    bijux_stages_fastq::fastq::registry()
        .iter()
        .find(|stage| stage.id == stage_id)
        .is_some_and(|stage| stage.affects_read_counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_core::{
        CommandSpecV1, ContainerImageRefV1, StageIO, StageId, StageVersion, ToolConstraints, ToolId,
    };

    #[test]
    fn polyx_warning_is_stage_wide() {
        let plan = StagePlanV1 {
            stage_id: StageId("fastq.trim".to_string()),
            stage_version: StageVersion(1),
            tool_id: ToolId("cutadapt".to_string()),
            tool_version: "1.0.0".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: Vec::new(),
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: Vec::new(),
                outputs: Vec::new(),
            },
            out_dir: std::path::PathBuf::from("out"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({
                "paired_mode": "single_end",
                "threads": 1,
                "min_len": 0,
                "adapter_policy": "none"
            }),
            aux_images: std::collections::BTreeMap::new(),
        };
        let params = serde_json::json!({
            "polyx_bank": {
                "preset": "illumina_twocolor"
            }
        });
        let warnings = warnings_for_plan(&plan, &params);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("polyx preset requested"));
    }
}
