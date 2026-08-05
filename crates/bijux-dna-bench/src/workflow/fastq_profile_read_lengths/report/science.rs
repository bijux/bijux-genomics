//! Owner: bijux-dna-bench
//! Contract-aware scientific and resource derivation for FASTQ read-length reports.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkExecutionSpec, BenchmarkMetricDistribution, BenchmarkScientificValue,
    BenchmarkStageCampaignSpec,
};
use bijux_dna_science::evidence::{summarize_fastq_read_length_histogram, FastqReadLengthSummary};
use serde::{Deserialize, Serialize};

use super::{
    hash_json, read_json, BUNDLE_SCHEMA, MAX_REPORT_INPUT_BYTES, METRICS_SCHEMA, STAGE_ID,
};
use crate::workflow::stage_report::{distribution, ResourceObservation};

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

#[derive(Debug, Clone)]
pub(super) struct ScientificObservation {
    pub(super) values: BTreeMap<String, BenchmarkScientificValue>,
    pub(super) input_bytes: u64,
    pub(super) output_bytes: u64,
    pub(super) read_count: u64,
    pub(super) normalized_bundle_sha256: String,
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
) -> Result<(FastqReadLengthSummary, u64, String)> {
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
        let sample = summarize_fastq_read_length_histogram(&histogram)?;
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
        summarize_fastq_read_length_histogram(&aggregate)?,
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

fn normalized_bundle_sha256(bundle: &ReadLengthBundle) -> Result<String> {
    let mut normalized = bundle.clone();
    normalized.results.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    for row in &mut normalized.results {
        row.histogram_artifact = format!("repetitions/sample-{}.histogram.tsv", row.sample_id);
    }
    hash_json(&normalized)
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

fn number(value: u64) -> BenchmarkScientificValue {
    BenchmarkScientificValue::Number(value as f64)
}
