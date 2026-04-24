use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_analyze::{BenchmarkContext, FastqDeltaMetrics};
use bijux_dna_core::prelude::measure::SeqkitMetrics;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
use bijux_dna_infra::{bench_base_dir, bench_tools_dir, hash_file_sha256};
use bijux_dna_planner_fastq::stage_api::bench_dir_name;
use bijux_dna_planner_fastq::stage_api::observer::{input_fastq_stats, parse_seqkit_stats};
use bijux_dna_runner::backend::docker::executor::resolve_image_for_run;
use bijux_dna_runner::step_runner::execute_observer_command;
use serde_json::Value;

use crate::support::benchmark_runtime::ensure_bench_runner;

pub(crate) struct TrimBenchInputs {
    pub(crate) runner: RuntimeKind,
    pub(crate) r1: PathBuf,
    pub(crate) input_hash: String,
    pub(crate) input_stats: SeqkitMetrics,
    pub(crate) bench_dir: PathBuf,
    pub(crate) tools_root: PathBuf,
}

pub(crate) fn prepare_trim_bench<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner_override: Option<RuntimeKind>,
    sample_id: &str,
    out: &Path,
    r1: &Path,
    stage_id: &bijux_dna_core::ids::StageId,
) -> Result<TrimBenchInputs> {
    let runner = ensure_bench_runner(platform, runner_override)?;
    let bench_dir_name = bench_dir_name(stage_id)
        .ok_or_else(|| anyhow!("bench dir missing for {}", stage_id.as_str()))?;
    let bench_dir = bench_base_dir(out, bench_dir_name, sample_id);
    let tools_root = bench_tools_dir(out, bench_dir_name, sample_id);
    bijux_dna_infra::ensure_dir(&bench_dir).context("create bench output dir")?;
    bijux_dna_infra::ensure_dir(&tools_root).context("create tools output dir")?;

    let r1 = r1.canonicalize().context("resolve r1 path")?;
    let input_hash = hash_file_sha256(&r1).context("hash trim input")?;
    let input_stats = observe_fastq_stats(catalog, platform, runner, &r1)?;

    Ok(TrimBenchInputs { runner, r1, input_hash, input_stats, bench_dir, tools_root })
}

pub(crate) fn observe_fastq_stats<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    runner: RuntimeKind,
    fastq: &Path,
) -> Result<SeqkitMetrics> {
    let fastq_dir =
        fastq.parent().ok_or_else(|| anyhow!("fastq has no parent: {}", fastq.display()))?;
    let seqkit_tool = catalog
        .get(bijux_dna_planner_fastq::stage_api::TOOL_SEQKIT)
        .ok_or_else(|| anyhow!("seqkit missing from images catalog"))?;
    let seqkit_image = resolve_image_for_run(seqkit_tool, platform)?;
    let stats_spec = input_fastq_stats(fastq_dir, fastq)?;
    let stats_output = execute_observer_command(
        &seqkit_image.full_name,
        stats_spec.mount_dir.as_path(),
        &stats_spec.args,
        runner,
    )?;
    if stats_output.exit_code != 0 {
        return Err(anyhow!("seqkit stats failed: {}", stats_output.stderr));
    }
    parse_seqkit_stats(&stats_output.stdout)
}

pub(crate) fn require_existing_benchmark_output<'a>(
    path: &'a Path,
    artifact_label: &str,
) -> Result<&'a Path> {
    if path.exists() {
        Ok(path)
    } else {
        Err(anyhow!("expected benchmark output `{artifact_label}` at {}", path.display()))
    }
}

pub(crate) fn derive_trim_delta(
    before: &SeqkitMetrics,
    after: &SeqkitMetrics,
) -> FastqDeltaMetrics {
    FastqDeltaMetrics {
        read_retention: ratio_u64(after.reads, before.reads),
        base_retention: ratio_u64(after.bases, before.bases),
        mean_q_delta: after.mean_q - before.mean_q,
        gc_delta: after.gc_percent - before.gc_percent,
    }
}

pub(crate) fn build_benchmark_context(
    tool: &str,
    tool_version: String,
    image_digest: String,
    runner: RuntimeKind,
    platform: &PlatformSpec,
    input_hash: String,
    parameters: Value,
) -> BenchmarkContext {
    BenchmarkContext {
        tool: tool.to_string(),
        tool_version,
        image_digest,
        runner: runner.to_string(),
        platform: platform.name.clone(),
        input_hash,
        parameters: parameters.into(),
    }
}

pub(crate) fn benchmark_image_identity(tool_spec: &ToolExecutionSpecV1) -> String {
    tool_spec.image.digest.clone().unwrap_or_else(|| tool_spec.image.image.clone())
}

#[allow(dead_code)]
pub(crate) fn infer_udg_classification(input: &Path) -> String {
    if let Ok(configured) = std::env::var("BIJUX_UDG_CLASSIFICATION") {
        let normalized = configured.trim().to_ascii_lowercase();
        if matches!(normalized.as_str(), "udg" | "partial" | "non_udg") {
            return normalized;
        }
    }
    let stem = input
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if stem.contains("partial_udg") || stem.contains("partial-udg") {
        "partial".to_string()
    } else if stem.contains("udg") {
        "udg".to_string()
    } else {
        "non_udg".to_string()
    }
}

#[allow(dead_code)]
pub(crate) fn terminal_damage_profile(path: &Path) -> Result<Value> {
    let mut ct_events = 0_u64;
    let mut ga_events = 0_u64;
    let mut seen = 0_u64;
    let mut five_prime: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut three_prime: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut lines = open_fastq_lines(path)?;
    while let (Some(_h), Some(seq), Some(_plus), Some(_qual)) =
        (lines.next(), lines.next(), lines.next(), lines.next())
    {
        let seq = seq.trim().to_ascii_uppercase();
        if seq.len() < 2 {
            continue;
        }
        let first = seq.chars().next().unwrap_or('N');
        let last = seq.chars().next_back().unwrap_or('N');
        *five_prime.entry(first.to_string()).or_insert(0) += 1;
        *three_prime.entry(last.to_string()).or_insert(0) += 1;
        if seq.starts_with("CT") {
            ct_events += 1;
        }
        if seq.ends_with("GA") {
            ga_events += 1;
        }
        seen += 1;
        if seen >= 200_000 {
            break;
        }
    }
    let denom = u64_to_f64(ct_events + ga_events);
    let asymmetry =
        if denom > 0.0 { (u64_to_f64(ct_events) - u64_to_f64(ga_events)) / denom } else { 0.0 };
    Ok(serde_json::json!({
        "reads_profiled": seen,
        "terminal_base_composition_5p": five_prime,
        "terminal_base_composition_3p": three_prime,
        "ct_events": ct_events,
        "ga_events": ga_events,
        "ct_ga_asymmetry": asymmetry,
    }))
}

pub(crate) fn json_string(value: Option<&Value>, key: &str) -> Option<String> {
    value.and_then(|ctx| ctx.get(key)).and_then(Value::as_str).map(str::to_string)
}

fn ratio_u64(num: u64, denom: u64) -> f64 {
    if denom == 0 {
        0.0
    } else {
        u64_to_f64(num) / u64_to_f64(denom)
    }
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

#[allow(dead_code)]
fn open_fastq_lines(path: &Path) -> Result<Box<dyn Iterator<Item = String>>> {
    let file =
        std::fs::File::open(path).with_context(|| format!("open fastq {}", path.display()))?;
    if path.extension().and_then(|ext| ext.to_str()) == Some("gz") {
        let decoder = flate2::read::MultiGzDecoder::new(file);
        let reader = std::io::BufReader::new(decoder);
        let lines = reader
            .lines()
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("read gz fastq {}", path.display()))?;
        return Ok(Box::new(lines.into_iter()));
    }
    let reader = std::io::BufReader::new(file);
    let lines = reader
        .lines()
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("read fastq {}", path.display()))?;
    Ok(Box::new(lines.into_iter()))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::require_existing_benchmark_output;

    #[test]
    fn require_existing_benchmark_output_accepts_real_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("reads.fastq.gz");
        bijux_dna_infra::write_bytes(&path, b"fixture").expect("fixture");

        let resolved = require_existing_benchmark_output(&path, "corrected_reads_r1")
            .expect("existing output should be accepted");
        assert_eq!(resolved, path.as_path());
    }

    #[test]
    fn require_existing_benchmark_output_rejects_missing_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("missing.fastq.gz");
        let error = require_existing_benchmark_output(&path, "dedup_reads_r1")
            .expect_err("missing output should be rejected");

        assert!(error.to_string().contains("expected benchmark output `dedup_reads_r1`"));
    }
}
