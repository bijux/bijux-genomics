use std::collections::{BTreeMap, VecDeque};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use serde::Serialize;

use bijux_core::contract::canonical::parameters_json_canonicalization;
use bijux_core::contract::ContractVersion;
use bijux_core::prelude::hashing::{input_fingerprint, parameters_fingerprint};
use bijux_core::metrics::MetricsEnvelope;
use bijux_domain_fastq::metrics::*;
use bijux_domain_fastq::parse_effective_params;
use bijux_stage_contract::StagePlanV1;

#[derive(Debug, Default, Clone)]
pub struct FilterRemovalCounts {
    pub by_n: u64,
    pub by_entropy: u64,
    pub by_low_complexity: u64,
    pub by_kmer: u64,
    pub by_contaminant_kmer: u64,
    pub by_length: u64,
}

#[derive(Debug, Clone, Serialize)]
struct FastqcMetricsV2 {
    schema_version: String,
    source: String,
    per_base_quality: Option<PerBaseQualitySummary>,
    gc_distribution: Option<GcDistributionSummary>,
    adapter_content: Option<AdapterContentSummary>,
    duplication: Option<DuplicationSummary>,
    n_content: Option<NContentSummary>,
    kmer_content: Option<KmerContentSummary>,
}

#[derive(Debug, Clone, Serialize)]
struct PerBaseQualitySummary {
    mean_min: f64,
    mean_max: f64,
    mean_mean: f64,
    bases_below_q20: u64,
    bases_below_q30: u64,
}

#[derive(Debug, Clone, Serialize)]
struct GcDistributionSummary {
    mean_gc: f64,
    std_gc: f64,
    outlier: bool,
}

#[derive(Debug, Clone, Serialize)]
struct AdapterContentSummary {
    max_percent: f64,
    mean_percent: f64,
    adapters: Vec<AdapterSignal>,
}

#[derive(Debug, Clone, Serialize)]
struct AdapterSignal {
    name: String,
    max_percent: f64,
    mean_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
struct DuplicationSummary {
    unique_fraction: f64,
    duplication_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
struct NContentSummary {
    mean_percent: f64,
    max_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
struct KmerContentSummary {
    warning_count: u64,
    kmers: Vec<KmerSignal>,
}

#[derive(Debug, Clone, Serialize)]
struct KmerSignal {
    kmer: String,
    count: u64,
    percent: f64,
}

pub fn stage_metrics_for_plan(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let mut metrics = match plan.stage_id.as_str() {
        "fastq.trim" => {
            let stats = stats_for_paths(&[
                inputs.first().map(PathBuf::as_path),
                outputs.first().map(PathBuf::as_path),
            ])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
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
                stage_boundary: plan.stage_id.to_string(),
                conditions: retention_conditions_from_effective(
                    &plan.stage_id,
                    &plan.effective_params,
                    &plan.params,
                ),
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
            let removals =
                filter_removals_for_plan(plan.tool_id.as_str(), &plan.out_dir, &plan.params);
            filter_metrics_with_removals(
                &plan.stage_id,
                inputs,
                outputs,
                &plan.params,
                &plan.effective_params,
                &removals,
            )?
        }
        "fastq.merge" => {
            let stats = stats_for_paths(&[
                inputs.first().map(PathBuf::as_path),
                inputs.get(1).map(PathBuf::as_path),
                outputs.first().map(PathBuf::as_path),
                outputs.get(1).map(PathBuf::as_path),
                outputs.get(2).map(PathBuf::as_path),
            ])?;
            let r1 = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let r2 = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
            let merged = stats.get(2).copied().unwrap_or_else(zero_seqkit_metrics);
            let unmerged_r1 = stats.get(3).copied().unwrap_or_else(zero_seqkit_metrics);
            let unmerged_r2 = stats.get(4).copied().unwrap_or_else(zero_seqkit_metrics);
            let reads_unmerged = unmerged_r1.reads.min(unmerged_r2.reads);
            let min_reads = r1.reads.min(r2.reads);
            let merge_rate = if min_reads > 0 {
                f64_from_u64(merged.reads) / f64_from_u64(min_reads)
            } else {
                0.0
            };
            let bases_in = r1.bases.min(r2.bases);
            let mean_q_in = (r1.mean_q + r2.mean_q) / 2.0;
            let merge_q_delta = merged.mean_q - mean_q_in;
            serde_json::to_value(FastqMergeMetricsV1 {
                reads_in: min_reads,
                reads_out: merged.reads,
                bases_in,
                bases_out: merged.bases,
                pairs_in: Some(min_reads),
                pairs_out: Some(merged.reads),
                reads_r1: r1.reads,
                reads_r2: r2.reads,
                reads_merged: merged.reads,
                reads_unmerged,
                reads_discarded: 0,
                merge_rate,
                merge_q_delta,
            })?
        }
        "fastq.validate_pre" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
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
        "fastq.detect_adapters" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let out_dir = path_from_params(&plan.params, "out_dir")
                .unwrap_or_else(|| inputs.first().cloned().unwrap_or_default());
            let metrics = fastqc_metrics_v2_from_dir(&out_dir).or_else(|| {
                let subdir = out_dir.join("fastqc");
                fastqc_metrics_v2_from_dir(&subdir)
            });
            let adapter_content_max = metrics
                .as_ref()
                .and_then(|m| m.adapter_content.as_ref().map(|a| a.max_percent));
            let adapter_content_mean = metrics
                .as_ref()
                .and_then(|m| m.adapter_content.as_ref().map(|a| a.mean_percent));
            let duplication_rate = metrics
                .as_ref()
                .and_then(|m| m.duplication.as_ref().map(|d| d.duplication_rate));
            let n_rate = metrics
                .as_ref()
                .and_then(|m| m.n_content.as_ref().map(|n| n.mean_percent / 100.0));
            let kmer_warning_count = metrics
                .as_ref()
                .and_then(|m| m.kmer_content.as_ref().map(|k| k.warning_count));
            serde_json::to_value(FastqDetectAdaptersMetricsV1 {
                reads_in: input.reads,
                reads_out: input.reads,
                bases_in: input.bases,
                bases_out: input.bases,
                pairs_in,
                pairs_out,
                mean_q: input.mean_q,
                adapter_content_max,
                adapter_content_mean,
                duplication_rate,
                n_rate,
                kmer_warning_count,
            })?
        }
        "fastq.correct" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::to_value(FastqCorrectMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
                reads_corrected: output.reads,
                reads_uncorrected: input.reads.saturating_sub(output.reads),
                bases_corrected: output.bases,
                bases_uncorrected: input.bases.saturating_sub(output.bases),
            })?
        }
        "fastq.umi" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
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
                stage_boundary: plan.stage_id.to_string(),
                conditions: retention_conditions_from_effective(
                    &plan.stage_id,
                    &plan.effective_params,
                    &plan.params,
                ),
            };
            serde_json::to_value(FastqUmiMetricsV1 {
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
        "fastq.preprocess" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
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
                stage_boundary: plan.stage_id.to_string(),
                conditions: retention_conditions_from_effective(
                    &plan.stage_id,
                    &plan.effective_params,
                    &plan.params,
                ),
            };
            serde_json::to_value(FastqPreprocessMetricsV1 {
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
        "fastq.qc_post" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let out_dir = path_from_params(&plan.params, "out_dir")
                .unwrap_or_else(|| outputs.first().cloned().unwrap_or_default());
            let raw_dir = out_dir.join("fastqc_raw");
            let trimmed_dir = out_dir.join("fastqc_trimmed");
            let multiqc_report = out_dir.join("multiqc_report.html");
            let multiqc_data = out_dir.join("multiqc_data");
            let raw_metrics = fastqc_metrics_v2_from_dir(&raw_dir);
            let trimmed_metrics = fastqc_metrics_v2_from_dir(&trimmed_dir);
            let metrics_source = trimmed_metrics.as_ref().or(raw_metrics.as_ref());
            let adapter_content_max =
                metrics_source.and_then(|m| m.adapter_content.as_ref().map(|a| a.max_percent));
            let adapter_content_mean =
                metrics_source.and_then(|m| m.adapter_content.as_ref().map(|a| a.mean_percent));
            let duplication_rate =
                metrics_source.and_then(|m| m.duplication.as_ref().map(|d| d.duplication_rate));
            let n_rate =
                metrics_source.and_then(|m| m.n_content.as_ref().map(|n| n.mean_percent / 100.0));
            let kmer_warning_count =
                metrics_source.and_then(|m| m.kmer_content.as_ref().map(|k| k.warning_count));
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
                stage_boundary: plan.stage_id.to_string(),
                conditions: retention_conditions_from_effective(
                    &plan.stage_id,
                    &plan.effective_params,
                    &plan.params,
                ),
            };
            serde_json::to_value(FastqQcPostMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
                mean_q: Some(output.mean_q),
                mean_q_before: input.mean_q,
                mean_q_after: output.mean_q,
                delta_metrics: delta,
                retention,
                contamination_rate: Some(0.0),
                adapter_content_max,
                adapter_content_mean,
                duplication_rate,
                n_rate,
                kmer_warning_count,
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
        "fastq.stats_neutral" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
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
        "fastq.screen" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let report_path = path_from_params(&plan.params, "report")
                .or_else(|| outputs.first().cloned())
                .unwrap_or_else(|| PathBuf::from("screen_report.tsv"));
            let (contamination_rate, contamination_summary) = parse_screen_report(&report_path)?;
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
        _ => serde_json::json!({}),
    };
    if plan.stage_id.0.starts_with("fastq.") {
        if let Some(obj) = metrics.as_object_mut() {
            if !obj.contains_key("pairs_in") || !obj.contains_key("pairs_out") {
                let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
                if !obj.contains_key("pairs_in") {
                    obj.insert("pairs_in".to_string(), serde_json::to_value(pairs_in)?);
                }
                if !obj.contains_key("pairs_out") {
                    obj.insert("pairs_out".to_string(), serde_json::to_value(pairs_out)?);
                }
            }
        }
    }
    Ok(metrics)
}

/// Build a fully-formed metrics envelope for a stage plan.
///
/// # Errors
/// Returns an error if metrics cannot be computed or hashed.
pub fn build_metrics_envelope(
    plan: &StagePlanV1,
    input_paths: &[PathBuf],
    output_paths: &[PathBuf],
) -> Result<MetricsEnvelope<serde_json::Value>> {
    let metrics = stage_metrics_for_plan(plan, input_paths, output_paths)?;
    let mut input_hashes = Vec::new();
    for path in input_paths {
        if path.exists() {
            if let Ok(hash) = bijux_infra::hash_file_sha256(path) {
                input_hashes.push(hash);
            }
        }
    }
    let input_fingerprint = input_fingerprint(&input_hashes);
    let parameters_fingerprint = parameters_fingerprint(&plan.params)?;
    let parameters_json_normalized = parameters_json_canonicalization(&plan.params);
    let image_digest = plan
        .image
        .digest
        .clone()
        .unwrap_or_else(|| plan.image.image.clone());
    Ok(MetricsEnvelope {
        schema_version: "bijux.metrics_envelope.v2".to_string(),
        contract_version: ContractVersion::v1(),
        stage_id: plan.stage_id.0.to_string(),
        stage_version: plan.stage_version.0,
        tool_id: plan.tool_id.0.to_string(),
        tool_version: plan.tool_version.clone(),
        image_digest,
        parameters_fingerprint,
        input_fingerprint,
        parameters_json_normalized,
        input_hashes,
        metrics,
    })
}

pub fn retention_conditions_from_effective(
    stage_id: &bijux_core::ids::StageId,
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

pub fn bank_refs_from_params(params: &serde_json::Value) -> serde_json::Value {
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

pub fn filter_removals_for_plan(
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

pub fn stats_or_zero(
    path: Option<&Path>,
) -> Result<bijux_core::prelude::measure::SeqkitMetrics> {
    if let Some(path) = path {
        if path.exists() {
            if path.is_dir() {
                return Ok(bijux_core::prelude::measure::SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                });
            }
            if std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) == 0 {
                return Ok(bijux_core::prelude::measure::SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                });
            }
            return fastq_stats(path);
        }
    }
    Ok(bijux_core::prelude::measure::SeqkitMetrics {
        reads: 0,
        bases: 0,
        mean_q: 0.0,
        gc_percent: 0.0,
    })
}

fn stats_for_paths(
    paths: &[Option<&Path>],
) -> Result<Vec<bijux_core::prelude::measure::SeqkitMetrics>> {
    let tasks: Vec<(usize, Option<PathBuf>)> = paths
        .iter()
        .enumerate()
        .map(|(idx, path)| (idx, path.map(Path::to_path_buf)))
        .collect();
    if tasks.len() <= 1 || observer_jobs() == 1 {
        return tasks
            .into_iter()
            .map(|(_, path)| stats_or_zero(path.as_deref()))
            .collect();
    }
    let queue = Arc::new(Mutex::new(VecDeque::from(tasks)));
    let mut initial = Vec::with_capacity(paths.len());
    initial.resize_with(paths.len(), || None);
    let results: Arc<Mutex<Vec<Option<Result<bijux_core::prelude::measure::SeqkitMetrics>>>>> =
        Arc::new(Mutex::new(initial));
    let mut workers = Vec::new();
    let job_count = observer_jobs().min(paths.len());
    for _ in 0..job_count {
        let queue = Arc::clone(&queue);
        let results = Arc::clone(&results);
        workers.push(std::thread::spawn(move || loop {
            let next = {
                match queue.lock() {
                    Ok(mut queue) => queue.pop_front(),
                    Err(_) => None,
                }
            };
            let Some((idx, path)) = next else {
                break;
            };
            let value = stats_or_zero(path.as_deref());
            if let Ok(mut results) = results.lock() {
                results[idx] = Some(value);
            }
        }));
    }
    for worker in workers {
        let _ = worker.join();
    }
    let results = Arc::try_unwrap(results)
        .map_err(|_| anyhow!("observer results still shared"))?
        .into_inner()
        .unwrap_or_default();
    let mut out = Vec::with_capacity(results.len());
    for entry in results {
        let value = entry.unwrap_or_else(|| Err(anyhow!("observer result missing")))?;
        out.push(value);
    }
    Ok(out)
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

fn zero_seqkit_metrics() -> bijux_core::prelude::measure::SeqkitMetrics {
    bijux_core::prelude::measure::SeqkitMetrics {
        reads: 0,
        bases: 0,
        mean_q: 0.0,
        gc_percent: 0.0,
    }
}

fn observer_jobs() -> usize {
    std::env::var("BIJUX_OBSERVER_JOBS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map_or(2, |value| value.clamp(1, 32))
}

fn f64_from_u64(value: u64) -> f64 {
    value as f64
}

fn fastq_stats(path: &Path) -> Result<bijux_core::prelude::measure::SeqkitMetrics> {
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
    let mean_q = if reads > 0 {
        q_sum as f64 / reads as f64
    } else {
        0.0
    };
    let gc_percent = if bases > 0 {
        (gc as f64 / bases as f64) * 100.0
    } else {
        0.0
    };
    Ok(bijux_core::prelude::measure::SeqkitMetrics {
        reads,
        bases,
        mean_q,
        gc_percent,
    })
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

fn filter_metrics_with_removals(
    stage_id: &bijux_core::ids::StageId,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
    removals: &FilterRemovalCounts,
) -> Result<serde_json::Value> {
    let stats = stats_for_paths(&[
        inputs.first().map(PathBuf::as_path),
        outputs.first().map(PathBuf::as_path),
    ])?;
    let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
    let output = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
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

fn path_from_params(params: &serde_json::Value, name: &str) -> Option<PathBuf> {
    params
        .get(name)
        .and_then(|value| value.as_str())
        .map(PathBuf::from)
}

fn parse_screen_report(path: &Path) -> Result<(f64, serde_json::Value)> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("screen report missing: {}", path.display()))?;
    let mut entries = Vec::new();
    let mut unmapped_percent = None;
    let mut errors = Vec::new();
    for (idx, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            errors.push(format!("line {} has {} columns", idx + 1, parts.len()));
            continue;
        }
        let label = parts[0].trim().to_string();
        let percent_col = parts
            .last()
            .ok_or_else(|| anyhow!("screen report line {} missing percent", idx + 1))?;
        let percent_str = percent_col.trim().trim_end_matches('%');
        let percent = percent_str
            .parse::<f64>()
            .with_context(|| format!("screen report line {} percent parse", idx + 1))?;
        let label_lower = label.to_lowercase();
        if label_lower.contains("unmapped")
            || (label_lower.contains("no hit") && unmapped_percent.is_none())
        {
            unmapped_percent = Some(percent);
        }
        entries.push(serde_json::json!({
            "reference": label,
            "percent": percent,
        }));
    }
    if !errors.is_empty() {
        return Err(anyhow!("screen report parse errors: {}", errors.join("; ")));
    }
    if entries.is_empty() {
        return Ok((
            0.0,
            serde_json::json!({
                "schema_version": "bijux.screen_summary.v1",
                "entries": entries,
                "warning": "empty_report",
            }),
        ));
    }
    let contamination_rate = unmapped_percent.map_or(0.0, |value| (100.0 - value).max(0.0) / 100.0);
    Ok((
        contamination_rate,
        serde_json::json!({
            "schema_version": "bijux.screen_summary.v1",
            "entries": entries,
        }),
    ))
}

fn fastqc_metrics_v2_from_dir(dir: &Path) -> Option<FastqcMetricsV2> {
    let path = find_fastqc_data(dir)?;
    let raw = std::fs::read_to_string(path).ok()?;
    let modules = parse_fastqc_modules(&raw);

    let per_base_quality = modules
        .get("Per base sequence quality")
        .and_then(|lines| parse_per_base_quality(lines));
    let gc_distribution = modules
        .get("Per sequence GC content")
        .and_then(|lines| parse_gc_distribution(lines));
    let adapter_content = modules
        .get("Adapter Content")
        .and_then(|lines| parse_adapter_content(lines));
    let duplication = modules
        .get("Sequence Duplication Levels")
        .map(|lines| parse_duplication(lines));
    let n_content = modules
        .get("Per base N content")
        .and_then(|lines| parse_n_content(lines));
    let kmer_content = modules
        .get("Kmer Content")
        .map(|lines| parse_kmer_content(lines));

    Some(FastqcMetricsV2 {
        schema_version: "bijux.fastqc_metrics.v2".to_string(),
        source: dir.display().to_string(),
        per_base_quality,
        gc_distribution,
        adapter_content,
        duplication,
        n_content,
        kmer_content,
    })
}

fn find_fastqc_data(dir: &Path) -> Option<PathBuf> {
    let candidates = [
        dir.join("fastqc_data.txt"),
        dir.join("fastqc_data"),
        dir.join("fastqc_data.txt.gz"),
    ];
    candidates.into_iter().find(|candidate| candidate.exists())
}

fn parse_fastqc_modules(raw: &str) -> BTreeMap<String, Vec<String>> {
    let mut modules: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current = None;
    for line in raw.lines() {
        if line.starts_with(">>") {
            if line.starts_with(">>END_MODULE") {
                current = None;
            } else {
                let name = line
                    .trim_start_matches(">>")
                    .split('\t')
                    .next()
                    .unwrap_or("");
                if !name.is_empty() {
                    modules.insert(name.to_string(), Vec::new());
                    current = Some(name.to_string());
                }
            }
            continue;
        }
        if let Some(name) = &current {
            modules
                .entry(name.clone())
                .or_default()
                .push(line.to_string());
        }
    }
    modules
}

fn parse_per_base_quality(lines: &[String]) -> Option<PerBaseQualitySummary> {
    let mut means = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let mean = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if let Some(mean) = mean {
            means.push(mean);
        }
    }
    if means.is_empty() {
        return None;
    }
    let mean_min = means.iter().copied().fold(f64::INFINITY, f64::min);
    let mean_max = means.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    #[allow(clippy::cast_precision_loss)]
    let mean_mean = means.iter().sum::<f64>() / means.len() as f64;
    let bases_below_q20 = means.iter().filter(|v| **v < 20.0).count() as u64;
    let bases_below_q30 = means.iter().filter(|v| **v < 30.0).count() as u64;
    Some(PerBaseQualitySummary {
        mean_min,
        mean_max,
        mean_mean,
        bases_below_q20,
        bases_below_q30,
    })
}

fn parse_gc_distribution(lines: &[String]) -> Option<GcDistributionSummary> {
    let mut total = 0.0;
    let mut weighted_sum = 0.0;
    let mut counts = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let gc = parts.first().and_then(|v| v.parse::<f64>().ok());
        let count = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if let (Some(gc), Some(count)) = (gc, count) {
            total += count;
            weighted_sum += gc * count;
            counts.push(count);
        }
    }
    if total <= 0.0 {
        return None;
    }
    let mean_gc = weighted_sum / total;
    let mut var_sum = 0.0;
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let gc = parts.first().and_then(|v| v.parse::<f64>().ok());
        let count = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if let (Some(gc), Some(count)) = (gc, count) {
            var_sum += (gc - mean_gc).powi(2) * count;
        }
    }
    let std_gc = (var_sum / total).sqrt();
    #[allow(clippy::cast_precision_loss)]
    let mean_count = counts.iter().sum::<f64>() / counts.len() as f64;
    let mut count_var = 0.0;
    for count in &counts {
        count_var += (count - mean_count).powi(2);
    }
    #[allow(clippy::cast_precision_loss)]
    let count_std = (count_var / counts.len() as f64).sqrt();
    let outlier = counts
        .iter()
        .any(|count| *count > mean_count + (3.0 * count_std));
    Some(GcDistributionSummary {
        mean_gc,
        std_gc,
        outlier,
    })
}

fn parse_adapter_content(lines: &[String]) -> Option<AdapterContentSummary> {
    let mut header: Option<Vec<String>> = None;
    let mut per_adapter: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
            continue;
        }
        if header.is_none() && parts[0].to_lowercase().contains("position") {
            header = Some(parts.iter().map(std::string::ToString::to_string).collect());
            continue;
        }
        let Some(header) = header.as_ref() else {
            continue;
        };
        if parts.len() < header.len() {
            continue;
        }
        for (idx, name) in header.iter().enumerate().skip(1) {
            let value = parts
                .get(idx)
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            per_adapter.entry(name.clone()).or_default().push(value);
        }
    }
    if per_adapter.is_empty() {
        return None;
    }
    let mut adapters = Vec::new();
    let mut max_percent: f64 = 0.0;
    let mut sum = 0.0;
    let mut count = 0.0;
    for (name, values) in &per_adapter {
        if values.is_empty() {
            continue;
        }
        let local_max = values.iter().copied().fold(0.0, f64::max);
        #[allow(clippy::cast_precision_loss)]
        let local_mean = values.iter().sum::<f64>() / values.len() as f64;
        max_percent = max_percent.max(local_max);
        sum += values.iter().sum::<f64>();
        #[allow(clippy::cast_precision_loss)]
        {
            count += values.len() as f64;
        }
        adapters.push(AdapterSignal {
            name: name.clone(),
            max_percent: local_max,
            mean_percent: local_mean,
        });
    }
    let mean_percent = if count > 0.0 { sum / count } else { 0.0 };
    Some(AdapterContentSummary {
        max_percent,
        mean_percent,
        adapters,
    })
}

fn parse_duplication(lines: &[String]) -> DuplicationSummary {
    let mut unique_fraction = None;
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let level = parts.first().and_then(|v| v.parse::<u64>().ok());
        let percent = parts.get(1).and_then(|v| v.parse::<f64>().ok());
        if level == Some(1) {
            unique_fraction = percent.map(|v| v / 100.0);
            break;
        }
    }
    let unique_fraction = unique_fraction.unwrap_or(0.0);
    DuplicationSummary {
        unique_fraction,
        duplication_rate: (1.0 - unique_fraction).max(0.0),
    }
}

fn parse_n_content(lines: &[String]) -> Option<NContentSummary> {
    let mut values = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        if let Some(value) = parts.get(1).and_then(|v| v.parse::<f64>().ok()) {
            values.push(value);
        }
    }
    if values.is_empty() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let mean_percent = values.iter().sum::<f64>() / values.len() as f64;
    let max_percent = values.iter().copied().fold(0.0, f64::max);
    Some(NContentSummary {
        mean_percent,
        max_percent,
    })
}

fn parse_kmer_content(lines: &[String]) -> KmerContentSummary {
    let mut warning_count = 0;
    let mut kmers = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }
        let kmer = parts[0].to_string();
        let count = parts[1].parse::<u64>().unwrap_or(0);
        let percent = parts[2].parse::<f64>().unwrap_or(0.0);
        if percent > 0.0 {
            warning_count += 1;
        }
        kmers.push(KmerSignal {
            kmer,
            count,
            percent,
        });
    }
    KmerContentSummary {
        warning_count,
        kmers,
    }
}
