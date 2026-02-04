use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::{ResolvedImage, RunnerKind};
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::exec_helpers::{
    cleanup_execution, execution_memory_mb, run_filter_execution, run_merge_execution,
    run_multiqc_execution, run_tool_execution, run_validate_execution,
};
use crate::observer::{hash_file_sha256, Observer};
use crate::run_artifacts::{
    default_trace_ids, run_artifacts_dir_for_out, write_execution_logs_bounded, write_facts_jsonl,
    write_metrics_envelope, write_observability_manifest, write_plan_artifacts,
    write_progress_event_jsonl, write_runs_export_jsonl, write_stage_event_jsonl,
    write_stage_metrics_json, write_telemetry_event, write_tool_invocation_json,
};
use bijux_core::{
    parameters_json_canonicalization, AdapterBankProvenanceV1, ArtifactRef, BankRefV1, FactsRowV1,
    MetricContextV1, StageMetricsV1, StageObservabilityContextV1, StagePlanV1, ToolInvocationV1,
};

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

fn stage_version_i32(version: bijux_core::StageVersion) -> i32 {
    i32::try_from(version.0).unwrap_or(i32::MAX)
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

