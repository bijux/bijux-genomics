//! Owner: bijux-dna-bench
//! Contract-aware scientific and resource derivation for FASTQ read-length reports.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkExecutionSpec, BenchmarkMetricDistribution, BenchmarkScientificValue,
    BenchmarkStageCampaignSpec,
};
use serde::{Deserialize, Serialize};

use super::{
    hash_json, read_json, BUNDLE_SCHEMA, MAX_REPORT_INPUT_BYTES, METRICS_SCHEMA, STAGE_ID,
};

const SCIENTIFIC_FLOAT_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReadLengthBundle {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    results: Vec<ReadLengthBundleRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReadLengthBundleRow {
    sample_id: String,
    input_layout: String,
    file_count: u64,
    read_count: u64,
    minimum_read_length: u64,
    mean_read_length: f64,
    median_read_length: f64,
    maximum_read_length: u64,
    distinct_read_lengths: u64,
    histogram_artifact: String,
    histogram_sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadLengthMetrics {
    schema_version: String,
    sample_count: u64,
    file_count: u64,
    read_count: u64,
    minimum_read_length: u64,
    maximum_read_length: u64,
    exact_observation_mismatch_count: u64,
    histogram_mismatch_count: u64,
    input_bytes: u64,
    output_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ResourceObservation {
    wall_seconds: f64,
    user_seconds: f64,
    system_seconds: f64,
    peak_rss_kib: f64,
    filesystem_input_operations: f64,
    filesystem_output_operations: f64,
}

#[derive(Debug, Clone)]
pub(super) struct ScientificObservation {
    pub(super) values: BTreeMap<String, BenchmarkScientificValue>,
    pub(super) input_bytes: u64,
    pub(super) output_bytes: u64,
    pub(super) read_count: u64,
    pub(super) normalized_bundle_sha256: String,
}

#[derive(Debug, Clone, Copy)]
struct HistogramSummary {
    read_count: u64,
    minimum_read_length: u64,
    mean_read_length: f64,
    median_read_length: f64,
    maximum_read_length: u64,
    distinct_read_lengths: u64,
}

pub(super) fn derive_scientific_observation(
    spec: &BenchmarkExecutionSpec,
    campaign: &BenchmarkStageCampaignSpec,
    repetition: u32,
) -> Result<ScientificObservation> {
    let result_root = Path::new(&spec.outputs.result_root);
    let prefix = format!("repetitions/repetition-{repetition}");
    let metrics: ReadLengthMetrics =
        read_json(&result_root.join(format!("{prefix}.metrics.json")))?;
    if metrics.schema_version != METRICS_SCHEMA
        || metrics.sample_count == 0
        || metrics.file_count == 0
        || metrics.read_count == 0
        || metrics.input_bytes == 0
        || metrics.output_bytes == 0
        || metrics.exact_observation_mismatch_count != 0
        || metrics.histogram_mismatch_count != 0
    {
        bail!("FASTQ read-length metrics are not admissible");
    }
    let bundle_path = result_root.join(format!("{prefix}.{}", campaign.result_artifact));
    let bundle: ReadLengthBundle = read_json(&bundle_path)?;
    if bundle.schema_version != BUNDLE_SCHEMA
        || bundle.stage_id != STAGE_ID
        || bundle.tool_id != spec.identity.tool_id
        || bundle.results.len() != metrics.sample_count as usize
    {
        bail!("FASTQ read-length bundle identity differs from execution");
    }
    let (summary, file_count, histogram_sha256) =
        summarize_bundle_histograms(result_root, repetition, &bundle)?;
    if metrics.file_count != file_count
        || metrics.read_count != summary.read_count
        || metrics.minimum_read_length != summary.minimum_read_length
        || metrics.maximum_read_length != summary.maximum_read_length
    {
        bail!("FASTQ read-length metrics disagree with admitted histograms");
    }
    let normalized_bundle_sha256 = normalized_bundle_sha256(&bundle)?;
    let values = BTreeMap::from([
        ("file_count".to_string(), number(file_count)),
        ("read_count".to_string(), number(summary.read_count)),
        ("minimum_read_length".to_string(), number(summary.minimum_read_length)),
        (
            "mean_read_length".to_string(),
            BenchmarkScientificValue::Number(summary.mean_read_length),
        ),
        (
            "median_read_length".to_string(),
            BenchmarkScientificValue::Number(summary.median_read_length),
        ),
        ("maximum_read_length".to_string(), number(summary.maximum_read_length)),
        ("distinct_read_lengths".to_string(), number(summary.distinct_read_lengths)),
        ("histogram_sha256".to_string(), BenchmarkScientificValue::Text(histogram_sha256)),
    ]);
    Ok(ScientificObservation {
        values,
        input_bytes: metrics.input_bytes,
        output_bytes: metrics.output_bytes,
        read_count: summary.read_count,
        normalized_bundle_sha256,
    })
}

fn summarize_bundle_histograms(
    result_root: &Path,
    repetition: u32,
    bundle: &ReadLengthBundle,
) -> Result<(HistogramSummary, u64, String)> {
    let mut rows = bundle.results.iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    let mut seen = std::collections::BTreeSet::new();
    let mut aggregate = BTreeMap::new();
    let mut file_count = 0_u64;
    let mut digest_basis = Vec::new();
    for row in rows {
        if row.sample_id.trim().is_empty()
            || !seen.insert(row.sample_id.as_str())
            || !matches!(row.input_layout.as_str(), "single_end" | "paired_end")
            || row.file_count == 0
        {
            bail!("FASTQ read-length bundle contains an invalid sample row");
        }
        let expected_path =
            format!("repetitions/repetition-{repetition}.{}.histogram.tsv", row.sample_id);
        if row.histogram_artifact != expected_path {
            bail!("FASTQ histogram path differs from its repetition and sample identity");
        }
        let histogram_path = result_root.join(&row.histogram_artifact);
        let actual_sha256 = bijux_dna_infra::hash_file_sha256(&histogram_path)?;
        if actual_sha256 != row.histogram_sha256 {
            bail!("FASTQ histogram digest differs from its bundle record");
        }
        digest_basis.extend_from_slice(row.sample_id.as_bytes());
        digest_basis.push(b'\t');
        digest_basis.extend_from_slice(actual_sha256.as_bytes());
        digest_basis.push(b'\n');
        let histogram = read_histogram(&histogram_path, &row.sample_id)?;
        let sample = summarize_histogram(&histogram)?;
        if sample.read_count != row.read_count
            || sample.minimum_read_length != row.minimum_read_length
            || sample.maximum_read_length != row.maximum_read_length
            || sample.distinct_read_lengths != row.distinct_read_lengths
            || (sample.mean_read_length - row.mean_read_length).abs() > SCIENTIFIC_FLOAT_TOLERANCE
            || (sample.median_read_length - row.median_read_length).abs()
                > SCIENTIFIC_FLOAT_TOLERANCE
        {
            bail!("FASTQ bundle sample summary disagrees with its histogram");
        }
        file_count = file_count.checked_add(row.file_count).context("FASTQ file count overflow")?;
        for (length, count) in histogram {
            let total = aggregate.entry(length).or_insert(0_u64);
            *total = total.checked_add(count).context("FASTQ histogram count overflow")?;
        }
    }
    Ok((
        summarize_histogram(&aggregate)?,
        file_count,
        bijux_dna_infra::hash_bytes_sha256(&digest_basis),
    ))
}

fn read_histogram(path: &Path, sample_id: &str) -> Result<BTreeMap<u64, u64>> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)
        .with_context(|| format!("read FASTQ histogram {}", path.display()))?;
    let text = std::str::from_utf8(&bytes).context("FASTQ histogram is not UTF-8")?;
    let mut lines = text.lines();
    if lines.next() != Some("sample_id\tread_length\tcount") {
        bail!("FASTQ histogram header is invalid");
    }
    let mut histogram = BTreeMap::new();
    for line in lines {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 3 || fields[0] != sample_id {
            bail!("FASTQ histogram row identity is invalid");
        }
        let length = fields[1].parse::<u64>().context("parse FASTQ read length")?;
        let count = fields[2].parse::<u64>().context("parse FASTQ histogram count")?;
        if length == 0 || count == 0 || histogram.insert(length, count).is_some() {
            bail!("FASTQ histogram contains an invalid or duplicate row");
        }
    }
    if histogram.is_empty() {
        bail!("FASTQ histogram contains no observations");
    }
    Ok(histogram)
}

fn summarize_histogram(histogram: &BTreeMap<u64, u64>) -> Result<HistogramSummary> {
    let read_count = histogram.values().try_fold(0_u64, |total, count| {
        total.checked_add(*count).context("FASTQ read count overflow")
    })?;
    if read_count == 0 {
        bail!("FASTQ histogram has no reads");
    }
    let base_count = histogram.iter().try_fold(0_u128, |total, (length, count)| {
        total
            .checked_add(u128::from(*length) * u128::from(*count))
            .context("FASTQ base count overflow")
    })?;
    let lower_rank = read_count.div_ceil(2);
    let upper_rank = read_count.saturating_add(1).div_ceil(2);
    let mut cumulative = 0_u64;
    let mut lower = None;
    let mut upper = None;
    for (length, count) in histogram {
        cumulative = cumulative.checked_add(*count).context("FASTQ rank overflow")?;
        if lower.is_none() && cumulative >= lower_rank {
            lower = Some(*length);
        }
        if cumulative >= upper_rank {
            upper = Some(*length);
            break;
        }
    }
    let minimum = *histogram.first_key_value().context("FASTQ histogram is empty")?.0;
    let maximum = *histogram.last_key_value().context("FASTQ histogram is empty")?.0;
    Ok(HistogramSummary {
        read_count,
        minimum_read_length: minimum,
        mean_read_length: base_count as f64 / read_count as f64,
        median_read_length: (lower.context("FASTQ lower median is absent")? as f64
            + upper.context("FASTQ upper median is absent")? as f64)
            / 2.0,
        maximum_read_length: maximum,
        distinct_read_lengths: histogram.len() as u64,
    })
}

fn normalized_bundle_sha256(bundle: &ReadLengthBundle) -> Result<String> {
    let mut normalized = bundle.clone();
    normalized.results.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    for row in &mut normalized.results {
        row.histogram_artifact = format!("repetitions/sample-{}.histogram.tsv", row.sample_id);
    }
    hash_json(&normalized)
}

pub(super) fn read_resource_observation(path: &Path) -> Result<ResourceObservation> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 64 * 1024)
        .with_context(|| format!("read benchmark resources {}", path.display()))?;
    let text = std::str::from_utf8(&bytes).context("benchmark resource TSV is not UTF-8")?;
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() != 2
        || lines[0]
            != "wall_seconds\tuser_seconds\tsystem_seconds\tmax_rss_kb\tfilesystem_inputs\tfilesystem_outputs\texit_code"
    {
        bail!("benchmark resource TSV shape is invalid");
    }
    let fields = lines[1].split('\t').collect::<Vec<_>>();
    if fields.len() != 7 || fields[6] != "0" {
        bail!("benchmark resource TSV records an invalid execution");
    }
    let observation = ResourceObservation {
        wall_seconds: parse_nonnegative(fields[0], "wall seconds")?,
        user_seconds: parse_nonnegative(fields[1], "user seconds")?,
        system_seconds: parse_nonnegative(fields[2], "system seconds")?,
        peak_rss_kib: parse_nonnegative(fields[3], "peak RSS")?,
        filesystem_input_operations: parse_nonnegative(fields[4], "filesystem input operations")?,
        filesystem_output_operations: parse_nonnegative(fields[5], "filesystem output operations")?,
    };
    if observation.wall_seconds <= 0.0 || observation.peak_rss_kib <= 0.0 {
        bail!("benchmark resource TSV lacks wall time or peak RSS");
    }
    Ok(observation)
}

pub(super) fn performance_distributions(
    resources: &[ResourceObservation],
    science: &[ScientificObservation],
    input_bytes: u64,
) -> Result<BTreeMap<String, BenchmarkMetricDistribution>> {
    if resources.len() != science.len() || resources.len() < 3 {
        bail!("benchmark report requires aligned measured repetitions");
    }
    let wall = resources.iter().map(|row| row.wall_seconds).collect::<Vec<_>>();
    let cpu = resources.iter().map(|row| row.user_seconds + row.system_seconds).collect::<Vec<_>>();
    let input = input_bytes as f64;
    Ok(BTreeMap::from([
        ("wall_seconds".to_string(), distribution(&wall)?),
        ("cpu_seconds".to_string(), distribution(&cpu)?),
        (
            "cpu_utilization_percent".to_string(),
            distribution(
                &cpu.iter().zip(&wall).map(|(cpu, wall)| cpu / wall * 100.0).collect::<Vec<_>>(),
            )?,
        ),
        (
            "peak_rss_kib".to_string(),
            distribution(&resources.iter().map(|row| row.peak_rss_kib).collect::<Vec<_>>())?,
        ),
        (
            "filesystem_input_operations".to_string(),
            distribution(
                &resources.iter().map(|row| row.filesystem_input_operations).collect::<Vec<_>>(),
            )?,
        ),
        (
            "filesystem_output_operations".to_string(),
            distribution(
                &resources.iter().map(|row| row.filesystem_output_operations).collect::<Vec<_>>(),
            )?,
        ),
        ("input_bytes".to_string(), distribution(&vec![input; resources.len()])?),
        (
            "output_bytes".to_string(),
            distribution(&science.iter().map(|row| row.output_bytes as f64).collect::<Vec<_>>())?,
        ),
        (
            "throughput_input_bytes_per_second".to_string(),
            distribution(&wall.iter().map(|wall| input / wall).collect::<Vec<_>>())?,
        ),
        (
            "throughput_reads_per_second".to_string(),
            distribution(
                &science
                    .iter()
                    .zip(&wall)
                    .map(|(row, wall)| row.read_count as f64 / wall)
                    .collect::<Vec<_>>(),
            )?,
        ),
    ]))
}

fn distribution(values: &[f64]) -> Result<BenchmarkMetricDistribution> {
    if values.len() < 2 || values.iter().any(|value| !value.is_finite() || *value < 0.0) {
        bail!("benchmark distribution observations are invalid");
    }
    let minimum = values.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance =
        values.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    Ok(BenchmarkMetricDistribution {
        minimum,
        maximum,
        mean,
        sample_standard_deviation: variance.sqrt(),
    })
}

fn number(value: u64) -> BenchmarkScientificValue {
    BenchmarkScientificValue::Number(value as f64)
}

fn parse_nonnegative(value: &str, field: &str) -> Result<f64> {
    let value = value.parse::<f64>().with_context(|| format!("parse benchmark {field}"))?;
    if !value.is_finite() || value < 0.0 {
        bail!("benchmark {field} is invalid");
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{distribution, summarize_histogram};

    #[test]
    fn distributions_use_sample_standard_deviation() -> anyhow::Result<()> {
        let observed = distribution(&[1.0, 2.0, 3.0])?;
        assert_eq!(observed.minimum, 1.0);
        assert_eq!(observed.maximum, 3.0);
        assert_eq!(observed.mean, 2.0);
        assert_eq!(observed.sample_standard_deviation, 1.0);
        Ok(())
    }

    #[test]
    fn histogram_summary_preserves_even_median_and_weighted_mean() -> anyhow::Result<()> {
        let histogram = BTreeMap::from([(10, 1), (20, 2), (40, 1)]);
        let observed = summarize_histogram(&histogram)?;
        assert_eq!(observed.read_count, 4);
        assert_eq!(observed.minimum_read_length, 10);
        assert_eq!(observed.maximum_read_length, 40);
        assert_eq!(observed.distinct_read_lengths, 3);
        assert_eq!(observed.median_read_length, 20.0);
        assert_eq!(observed.mean_read_length, 22.5);
        Ok(())
    }
}
