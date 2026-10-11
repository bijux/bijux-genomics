//! Owner: bijux-dna-bench
//! Shared scientific and resource primitives for BAM observational reports.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkExecutionSpec, BenchmarkMetricDistribution, BenchmarkScientificValue,
};

use crate::workflow::stage_report::{distribution, ResourceObservation, MAX_REPORT_INPUT_BYTES};

#[derive(Debug, Clone)]
pub(crate) struct ScientificObservation {
    pub(crate) values: BTreeMap<String, BenchmarkScientificValue>,
    pub(crate) bam_input_bytes: u64,
    pub(crate) bam_index_input_bytes: u64,
    pub(crate) input_bytes: u64,
    pub(crate) output_bytes: u64,
    pub(crate) normalized_bundle_sha256: String,
}

pub(crate) fn read_numeric_summary(
    path: &Path,
    expected_metrics: &[&str],
) -> Result<BTreeMap<String, BenchmarkScientificValue>> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)
        .with_context(|| format!("read BAM observational summary {}", path.display()))?;
    let text = std::str::from_utf8(&bytes).context("BAM observational TSV is not UTF-8")?;
    let mut lines = text.lines();
    if lines.next() != Some("metric\tvalue") {
        bail!("BAM observational TSV header is invalid");
    }
    let mut values = BTreeMap::new();
    for line in lines {
        let (metric, value) = line.split_once('\t').context("BAM observational row is invalid")?;
        if !expected_metrics.contains(&metric) || value.contains('\t') {
            bail!("BAM observational metric is outside the contract");
        }
        let value = value.parse::<f64>().context("parse BAM observational metric")?;
        if !value.is_finite()
            || value < 0.0
            || values.insert(metric.to_string(), BenchmarkScientificValue::Number(value)).is_some()
        {
            bail!("BAM observational metric is invalid or duplicated");
        }
    }
    let observed = values.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected_metrics.iter().copied().collect::<BTreeSet<_>>();
    if observed != expected {
        bail!("BAM observational TSV lacks exact metric closure");
    }
    Ok(values)
}

pub(crate) fn numeric_value(
    summary: &BTreeMap<String, BenchmarkScientificValue>,
    metric: &str,
) -> Result<f64> {
    match summary.get(metric) {
        Some(BenchmarkScientificValue::Number(value)) => Ok(*value),
        _ => bail!("BAM observational TSV omits numeric {metric}"),
    }
}

pub(crate) fn validate_byte_counts(
    spec: &BenchmarkExecutionSpec,
    bam_input_bytes: u64,
    bam_index_input_bytes: u64,
    input_bytes: u64,
    output_bytes: u64,
    outputs: &[&Path],
) -> Result<()> {
    let expected_input = bam_input_bytes
        .checked_add(bam_index_input_bytes)
        .context("BAM observational input size overflow")?;
    let expected_output = outputs.iter().try_fold(0_u64, |total, path| {
        total.checked_add(regular_file_size(path)?).context("BAM observational output overflow")
    })?;
    if bam_input_bytes == 0
        || bam_index_input_bytes == 0
        || input_bytes != expected_input
        || bam_input_bytes != input_size(spec, "bam")?
        || bam_index_input_bytes != input_size(spec, "bam_index")?
        || output_bytes == 0
        || output_bytes != expected_output
    {
        bail!("BAM observational metrics contain invalid byte counts");
    }
    Ok(())
}

pub(super) fn performance_distributions(
    resources: &[ResourceObservation],
    science: &[ScientificObservation],
) -> Result<BTreeMap<String, BenchmarkMetricDistribution>> {
    if resources.len() != science.len() || resources.len() < 3 {
        bail!("benchmark report requires aligned measured repetitions");
    }
    Ok(BTreeMap::from([
        (
            "wall_seconds".to_string(),
            distribution(&resources.iter().map(|row| row.wall_seconds).collect::<Vec<_>>())?,
        ),
        (
            "cpu_seconds".to_string(),
            distribution(
                &resources
                    .iter()
                    .map(|row| row.user_seconds + row.system_seconds)
                    .collect::<Vec<_>>(),
            )?,
        ),
        (
            "peak_rss_kib".to_string(),
            distribution(&resources.iter().map(|row| row.peak_rss_kib).collect::<Vec<_>>())?,
        ),
        (
            "input_bytes".to_string(),
            distribution(&science.iter().map(|row| row.input_bytes as f64).collect::<Vec<_>>())?,
        ),
        (
            "output_bytes".to_string(),
            distribution(&science.iter().map(|row| row.output_bytes as f64).collect::<Vec<_>>())?,
        ),
    ]))
}

fn input_size(spec: &BenchmarkExecutionSpec, id: &str) -> Result<u64> {
    let input = spec
        .inputs
        .iter()
        .find(|input| input.id == id)
        .with_context(|| format!("BAM observational execution omits {id}"))?;
    regular_file_size(Path::new(&input.path))
}

fn regular_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect BAM observational evidence {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("BAM observational evidence is not a regular non-symlink file");
    }
    Ok(metadata.len())
}
