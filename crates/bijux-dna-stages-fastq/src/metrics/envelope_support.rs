use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::canonical::parameters_json_canonicalization;
use bijux_dna_core::contract::{ContractVersion, MetricProvenanceV1};
use bijux_dna_core::metrics::MetricsEnvelope;
use bijux_dna_core::prelude::hashing::{input_fingerprint, parameters_fingerprint};
use bijux_dna_domain_fastq::parse_effective_params;
use bijux_dna_stage_contract::StagePlanV1;
use flate2::read::GzDecoder;

use super::stage_metrics::stage_metrics_for_plan;
use crate::metrics::filters::{
    filter_removals_from_bbduk_stats, filter_removals_from_fastp, FilterRemovalCounts,
};

/// Build a fully-formed metrics envelope for a stage plan.
///
/// # Errors
/// Returns an error if metrics cannot be computed or hashed.
pub(crate) fn build_metrics_envelope(
    plan: &StagePlanV1,
    input_paths: &[PathBuf],
    output_paths: &[PathBuf],
) -> Result<MetricsEnvelope<serde_json::Value>> {
    let metrics = stage_metrics_for_plan(plan, input_paths, output_paths)?;
    let mut input_hashes = Vec::new();
    for path in input_paths {
        if path.exists() {
            if let Ok(hash) = bijux_dna_infra::hash_file_sha256(path) {
                input_hashes.push(hash);
            }
        }
    }
    input_hashes.sort();
    let input_fingerprint = input_fingerprint(&input_hashes);
    let parameters_fingerprint = parameters_fingerprint(&plan.params)?;
    let parameters_json_normalized = parameters_json_canonicalization(&plan.params);
    let image_digest = plan.image.digest.clone().unwrap_or_else(|| plan.image.image.clone());
    let tool_version = plan.tool_version.trim().to_string();
    Ok(MetricsEnvelope {
        schema_version: "bijux.metrics_envelope.v2".to_string(),
        contract_version: ContractVersion::v1(),
        stage_id: plan.stage_id.0.to_string(),
        stage_version: plan.stage_version.0,
        tool_id: plan.tool_id.0.to_string(),
        tool_version: tool_version.clone(),
        image_digest,
        parameters_fingerprint: parameters_fingerprint.clone(),
        input_fingerprint,
        parameters_json_normalized,
        input_hashes: input_hashes.clone(),
        metric_provenance: Some(MetricProvenanceV1 {
            run_id: "standalone".to_string(),
            stage_id: plan.stage_id.0.to_string(),
            tool_id: plan.tool_id.0.to_string(),
            tool_version,
            params_hash: parameters_fingerprint.clone(),
            input_artifact_hashes: input_hashes.clone(),
            manifest_hash: None,
        }),
        metrics,
    })
}

pub(crate) fn retention_conditions_from_effective(
    stage_id: &bijux_dna_core::ids::StageId,
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
        out.insert("condition".to_string(), serde_json::Value::String("effective".to_string()));
    } else {
        warning = Some("effective_params_missing");
        out.insert("parameters".to_string(), raw_params.clone());
        out.insert("condition".to_string(), serde_json::Value::String("unknown".to_string()));
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
        out.entry(key.to_string()).or_insert(serde_json::Value::Null);
    }
    if let Some(flag) = warning {
        out.insert("warning".to_string(), serde_json::Value::String(flag.to_string()));
    }
    serde_json::Value::Object(out)
}

pub(super) fn bank_refs_from_params(params: &serde_json::Value) -> serde_json::Value {
    let mut banks = serde_json::Map::new();
    for (key, field) in
        [("adapter", "adapter_bank"), ("polyx", "polyx_bank"), ("contaminant", "contaminant_bank")]
    {
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

pub(super) fn filter_removals_for_plan(
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

pub(super) fn stats_or_zero(
    path: Option<&Path>,
) -> Result<bijux_dna_core::prelude::measure::SeqkitMetrics> {
    if let Some(path) = path {
        if path.exists() {
            if path.is_dir() {
                return Ok(bijux_dna_core::prelude::measure::SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                });
            }
            if std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) == 0 {
                return Ok(bijux_dna_core::prelude::measure::SeqkitMetrics {
                    reads: 0,
                    bases: 0,
                    mean_q: 0.0,
                    gc_percent: 0.0,
                });
            }
            return fastq_stats(path);
        }
    }
    Ok(bijux_dna_core::prelude::measure::SeqkitMetrics {
        reads: 0,
        bases: 0,
        mean_q: 0.0,
        gc_percent: 0.0,
    })
}

pub(crate) fn stats_for_paths(
    paths: &[Option<&Path>],
) -> Result<Vec<bijux_dna_core::prelude::measure::SeqkitMetrics>> {
    let tasks: Vec<(usize, Option<PathBuf>)> =
        paths.iter().enumerate().map(|(idx, path)| (idx, path.map(Path::to_path_buf))).collect();
    if tasks.len() <= 1 || observer_jobs() == 1 {
        return tasks.into_iter().map(|(_, path)| stats_or_zero(path.as_deref())).collect();
    }
    let queue = Arc::new(Mutex::new(VecDeque::from(tasks)));
    let mut initial = Vec::with_capacity(paths.len());
    initial.resize_with(paths.len(), || None);
    let results: Arc<Mutex<Vec<Option<Result<bijux_dna_core::prelude::measure::SeqkitMetrics>>>>> =
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
    let results = finalize_observer_results(results)?;
    let mut out = Vec::with_capacity(results.len());
    for entry in results {
        let value = entry.unwrap_or_else(|| Err(anyhow!("observer result missing")))?;
        out.push(value);
    }
    Ok(out)
}

fn finalize_observer_results(
    results: Arc<Mutex<Vec<Option<Result<bijux_dna_core::prelude::measure::SeqkitMetrics>>>>>,
) -> Result<Vec<Option<Result<bijux_dna_core::prelude::measure::SeqkitMetrics>>>> {
    Arc::try_unwrap(results)
        .map_err(|_| anyhow!("observer results still shared"))?
        .into_inner()
        .map_err(|_| anyhow!("observer results lock poisoned"))
}

type LengthGcDistributions = (Vec<(u64, u64)>, Vec<(u8, u64)>);

pub(super) fn distributions_for_path(path: Option<&Path>) -> Result<LengthGcDistributions> {
    let Some(path) = path else {
        return Ok((Vec::new(), Vec::new()));
    };
    if !path.exists() || path.is_dir() {
        return Ok((Vec::new(), Vec::new()));
    }
    let file = std::fs::File::open(path).context("open fastq for distributions")?;
    let reader: Box<dyn std::io::Read> = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };
    let mut length_hist = std::collections::BTreeMap::<u64, u64>::new();
    let mut gc_hist = std::collections::BTreeMap::<u8, u64>::new();
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next() {
        let header = line?;
        if header.is_empty() {
            continue;
        }
        let seq = lines.next().ok_or_else(|| anyhow!("fastq missing sequence line"))??;
        let _plus = lines.next().ok_or_else(|| anyhow!("fastq missing plus line"))??;
        let _qual = lines.next().ok_or_else(|| anyhow!("fastq missing quality line"))??;
        let len = u64_from_usize(seq.len());
        *length_hist.entry(len).or_insert(0) += 1;
        let gc = u64_from_usize(
            seq.bytes().filter(|base| matches!(base, b'G' | b'g' | b'C' | b'c')).count(),
        );
        let gc_pct = rounded_percent(gc, len);
        *gc_hist.entry(gc_pct).or_insert(0) += 1;
    }
    Ok((length_hist.into_iter().collect(), gc_hist.into_iter().collect()))
}

pub(crate) fn pair_counts_from_paths(
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

pub(crate) fn zero_seqkit_metrics() -> bijux_dna_core::prelude::measure::SeqkitMetrics {
    bijux_dna_core::prelude::measure::SeqkitMetrics {
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

fn u64_from_usize(value: usize) -> u64 {
    match u64::try_from(value) {
        Ok(value) => value,
        Err(error) => panic!("usize must fit into u64 on supported targets: {error}"),
    }
}

fn rounded_percent(numerator: u64, denominator: u64) -> u8 {
    if denominator == 0 {
        return 0;
    }
    let rounded = numerator.saturating_mul(100).saturating_add(denominator / 2) / denominator;
    match u8::try_from(rounded) {
        Ok(value) => value,
        Err(error) => panic!("rounded percent must fit into u8: {error}"),
    }
}

#[allow(clippy::cast_precision_loss)]
pub(crate) fn f64_from_u64(value: u64) -> f64 {
    value as f64
}

fn fastq_stats(path: &Path) -> Result<bijux_dna_core::prelude::measure::SeqkitMetrics> {
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
        let seq = lines.next().ok_or_else(|| anyhow!("fastq missing sequence line"))??;
        let _plus = lines.next().ok_or_else(|| anyhow!("fastq missing plus line"))??;
        let qual = lines.next().ok_or_else(|| anyhow!("fastq missing quality line"))??;
        reads += 1;
        let seq_bytes = seq.as_bytes();
        bases += u64_from_usize(seq_bytes.len());
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
    let mean_q = if reads > 0 { f64_from_u64(q_sum) / f64_from_u64(reads) } else { 0.0 };
    let gc_percent = if bases > 0 { (f64_from_u64(gc) / f64_from_u64(bases)) * 100.0 } else { 0.0 };
    Ok(bijux_dna_core::prelude::measure::SeqkitMetrics { reads, bases, mean_q, gc_percent })
}

pub(super) fn path_from_params(params: &serde_json::Value, name: &str) -> Option<PathBuf> {
    params.get(name).and_then(|value| value.as_str()).map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::finalize_observer_results;
    use std::sync::{Arc, Mutex};

    #[test]
    fn finalize_observer_results_rejects_poisoned_mutex() {
        let results = Arc::new(Mutex::new(Vec::new()));
        let poisoned = Arc::clone(&results);
        let _ = std::thread::spawn(move || {
            let _guard =
                poisoned.lock().unwrap_or_else(|err| panic!("lock poisoned unexpectedly: {err}"));
            panic!("poison observer results");
        })
        .join();

        let Err(error) = finalize_observer_results(results) else {
            panic!("poisoned observer results must fail");
        };
        assert!(error.to_string().contains("observer results lock poisoned"));
    }
}
