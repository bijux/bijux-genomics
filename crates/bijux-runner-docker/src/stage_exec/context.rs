use std::collections::{BTreeMap, VecDeque};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_env_runtime::api::{ResolvedImage, RunnerKind};
use chrono::Utc;
use flate2::read::GzDecoder;
use tracing::info;
use uuid::Uuid;

use crate::primitives::{
    cleanup_execution, execution_memory_mb, hash_file_sha256, run_filter_execution,
    run_merge_execution, run_multiqc_execution, run_tool_execution, run_validate_execution,
};
use crate::observer::{
    parse_contamination_json, parse_damageprofiler_json, parse_mosdepth_summary,
    parse_pydamage_json, parse_preseq_estimates, parse_samtools_flagstat, parse_samtools_stats,
    parse_sex_json, Observer,
};
use bijux_engine::services::run_artifacts::{
    default_trace_ids, run_artifacts_dir_for_out, write_effective_adapters_from_provenance,
    write_execution_logs_bounded, write_facts_jsonl, write_filter_report_v1, write_merge_report_v1,
    write_metrics_envelope, write_observability_manifest, write_plan_artifacts,
    write_progress_event_jsonl, write_qc_post_report_v1, write_retention_report_v1,
    write_runs_export_jsonl, write_stage_event_jsonl, write_stage_metrics_json,
    write_stage_report_v1, write_telemetry_event, write_tool_invocation_json, write_trim_report_v1,
    write_validate_report_v1,
};
use bijux_core::run_index::{insert_stage_row, StageIndexRow};
use bijux_core::{
    parameters_json_canonicalization, AdapterBankProvenanceV1, ArtifactRef, BankRefV1, FactsRowV1,
    MetricContextV1, StageMetricsV1, StageObservabilityContextV1, StagePlanV1, ToolInvocationV1,
};
use bijux_domain_fastq::{evaluate_invariants, thresholds_from_env};

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

#[derive(Debug, Clone, serde::Serialize)]
struct FastqcMetricsV2 {
    schema_version: String,
    source: String,
    per_base_quality: Option<PerBaseQualitySummary>,
    gc_distribution: Option<GcDistributionSummary>,
    adapter_content: Option<AdapterContentSummary>,
    duplication: Option<DuplicationSummary>,
    n_content: Option<NContentSummary>,
    kmer_content: Option<KmerSummary>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct PerBaseQualitySummary {
    mean_min: f64,
    mean_max: f64,
    mean_mean: f64,
    bases_below_q20: u64,
    bases_below_q30: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
struct GcDistributionSummary {
    mean_gc: f64,
    std_gc: f64,
    outlier: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AdapterContentSummary {
    max_percent: f64,
    mean_percent: f64,
    adapters: Vec<AdapterSignal>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AdapterSignal {
    name: String,
    max_percent: f64,
    mean_percent: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
struct DuplicationSummary {
    unique_fraction: f64,
    duplication_rate: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
struct NContentSummary {
    mean_percent: f64,
    max_percent: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
struct KmerSummary {
    warning_count: u64,
    top_kmer: Option<String>,
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

fn parse_fastqc_modules(raw: &str) -> BTreeMap<String, Vec<String>> {
    let mut modules = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut buffer: Vec<String> = Vec::new();
    for line in raw.lines() {
        if line.starts_with(">>END_MODULE") {
            if let Some(name) = current.take() {
                modules.insert(name, std::mem::take(&mut buffer));
            }
            continue;
        }
        if line.starts_with(">>") {
            if let Some(name) = current.take() {
                modules.insert(name, std::mem::take(&mut buffer));
            }
            let name = line
                .trim_start_matches(">>")
                .split('\t')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            current = Some(name);
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if current.is_some() {
            buffer.push(line.to_string());
        }
    }
    if let Some(name) = current.take() {
        modules.insert(name, buffer);
    }
    modules
}
