use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::{ResolvedImage, RunnerKind};
use chrono::Utc;
use flate2::read::GzDecoder;
use tracing::info;
use uuid::Uuid;

use crate::api::{
    cleanup_execution, execution_memory_mb, hash_file_sha256, run_merge_execution,
    run_multiqc_execution, run_tool_execution, run_validate_execution,
};
use crate::services::observer::Observer;
use crate::services::run_artifacts::{
    default_trace_ids, params_hash, run_artifacts_dir_for_out, write_facts_jsonl,
    write_merge_report_v1, write_metrics_envelope, write_observability_manifest,
    write_plan_artifacts, write_progress_event_jsonl, write_retention_report_v1,
    write_runs_export_jsonl, write_stage_event_jsonl, write_stage_metrics_json,
    write_stage_report_v1, write_telemetry_event, write_tool_invocation_json, write_trim_report_v1,
    write_validate_report_v1,
};
use bijux_core::run_index::{insert_stage_row, StageIndexRow};
use bijux_core::{
    parameters_json_canonicalization, AdapterBankProvenanceV1, BankRefV1, FactsRowV1,
    FastqCorrectMetricsV1, FastqDeltaMetricsV1, FastqFilterMetricsV1, FastqMergeMetricsV1,
    FastqPreprocessMetricsV1, FastqTrimMetricsV1, FastqUmiMetricsV1, FastqValidateMetricsV1,
    MetricContextV1, RetentionReportMetricV1, StageMetricsV1, StageObservabilityContextV1,
    StagePlanV1, ToolInvocationV1,
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

fn retention_conditions_from_params(params: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "min_len": params.get("min_len"),
        "q": params.get("q"),
        "merge_policy": params.get("merge_policy"),
        "adapter_policy": params.get("adapter_policy"),
        "polyx_policy": params.get("polyx_policy"),
        "contaminant_policy": params.get("contaminant_policy"),
        "banks": bank_refs_from_params(params),
        "parameters": params.clone(),
    })
}

fn tool_supports_polyx(tool_id: &str) -> bool {
    matches!(tool_id, "fastp")
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
    warnings
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
                conditions: retention_conditions_from_params(params),
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
        "fastq.filter" => {
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
                conditions: retention_conditions_from_params(params),
            };
            serde_json::to_value(FastqFilterMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                reads_dropped: input.reads.saturating_sub(output.reads),
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
        "fastq.qc_post" | "fastq.screen" | "fastq.stats_neutral" => {
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
    emit_event(&bijux_core::TelemetryEventV1 {
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
                let fastqc_dir = plan.out_dir.join("fastqc");
                std::fs::create_dir_all(&fastqc_dir)?;
                let fastqc_container = format!("bijux-stage-fastqc-{}", Uuid::new_v4());
                let fastqc_exec = run_validate_execution(
                    "fastqc",
                    &fastqc_image,
                    r1_dir,
                    r1,
                    &fastqc_dir,
                    &fastqc_container,
                )?;
                cleanup_execution(&fastqc_container)?;
                if fastqc_exec.exit_code != 0 {
                    return Err(anyhow!("fastqc exit code {}", fastqc_exec.exit_code));
                }
                let exec =
                    run_multiqc_execution(&image, &fastqc_dir, &plan.out_dir, &container_name)?;
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
        let stage_metrics = stage_metrics_for_plan(
            plan.stage_id.0.as_str(),
            &input_paths,
            &outputs,
            &canonical_params,
        )?;
        let invocation = ToolInvocationV1 {
            schema_version: "bijux.tool_invocation.v1".to_string(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            tool_version: plan.tool_version.clone(),
            image_digest: image_digest.clone(),
            runner_kind: runner.to_string(),
            platform: std::env::var("BIJUX_PLATFORM").unwrap_or_else(|_| "unknown".to_string()),
            parameters_json: canonical_params.clone(),
            parameters_json_normalized: bijux_core::parameters_json_canonicalization(
                &canonical_params,
            ),
            adapter_bank: adapter_bank_from_params(&canonical_params),
            banks: banks_json.clone(),
            bank_assets: bank_assets.clone(),
            resources: plan.resources.clone(),
            environment: std::env::vars().collect::<BTreeMap<String, String>>(),
            input_hashes: input_hashes.clone(),
            output_hashes: output_hashes.clone(),
        };
        let tool_invocation_path =
            write_tool_invocation_json(&run_artifacts_dir, &plan.stage_id.0, &invocation)?;
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
        let metrics_path = run_artifacts_dir.join("metrics.json");
        let facts_row_id = format!("{}:{}:{}", run_id, plan.stage_id.0, plan.tool_id.0);
        let mut subreports: Vec<PathBuf> = Vec::new();
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
            subreports.push(report_path);
        }
        let warnings = warnings_for_plan(plan, &canonical_params);
        let stage_report_path = write_stage_report_v1(
            &run_artifacts_dir,
            &plan.stage_id.0,
            stage_version_i32(plan.stage_version),
            &plan.tool_id.0,
            &plan.tool_version,
            &metrics_path,
            &plan_artifacts.effective_config_path,
            Some(&facts_row_id),
            &outputs,
            &subreports,
            &[],
            &warnings,
            &[],
        )?;
        let (reads_in, reads_out, bases_in, bases_out, pairs_in, pairs_out) =
            extract_io_deltas(&stage_metrics);
        let retention_report_path = if is_retention_stage(&plan.stage_id.0) {
            retention_counts_for_plan(&plan.stage_id.0, &input_paths, &outputs)?.map(|counts| {
                write_retention_report_v1(
                    &run_artifacts_dir,
                    &plan.stage_id.0,
                    &plan.tool_id.0,
                    &plan.tool_version,
                    &retention_conditions_from_params(&canonical_params),
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
                }),
            },
        );
        write_facts_jsonl(
            &run_artifacts_dir.join("dashboard").join("facts.jsonl"),
            &FactsRowV1 {
                schema_version: "bijux.facts_row.v1".to_string(),
                run_id: run_id.clone(),
                stage_id: plan.stage_id.0.clone(),
                tool_id: plan.tool_id.0.clone(),
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
                }),
                artifacts: serde_json::json!({
                    "metrics_envelope": metrics_envelope_path.display().to_string(),
                    "stage_report": stage_report_path.display().to_string(),
                    "retention_report": retention_report_path.as_ref().map(|path| path.display().to_string()),
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
        if output.exists() {
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
