use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;

use bijux_core::contract::canonical::parameters_json_canonicalization;
use bijux_core::contract::ContractVersion;
use bijux_core::metrics::MetricsEnvelope;
use bijux_core::prelude::hashing::{input_fingerprint, parameters_fingerprint};
use bijux_domain_fastq::metrics::*;
use bijux_domain_fastq::parse_effective_params;
use bijux_stage_contract::StagePlanV1;

mod fastqc;
mod filters;

use fastqc::fastqc_metrics_v2_from_dir;
use filters::{
    filter_metrics_with_removals, filter_removals_from_bbduk_stats, filter_removals_from_fastp,
    parse_screen_report, FilterRemovalCounts,
};

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

pub fn stats_or_zero(path: Option<&Path>) -> Result<bijux_core::prelude::measure::SeqkitMetrics> {
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

fn path_from_params(params: &serde_json::Value, name: &str) -> Option<PathBuf> {
    params
        .get(name)
        .and_then(|value| value.as_str())
        .map(PathBuf::from)
}
