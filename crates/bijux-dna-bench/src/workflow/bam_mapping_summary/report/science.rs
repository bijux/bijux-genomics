//! Owner: bijux-dna-bench
//! Contract-aware scientific and resource derivation for BAM mapping-summary reports.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkExecutionSpec, BenchmarkMetricDistribution, BenchmarkScientificValue,
};
use serde::Deserialize;

use super::{BUNDLE_SCHEMA, STAGE_ID};
use crate::workflow::stage_report::{
    distribution, read_json, ResourceObservation, MAX_REPORT_INPUT_BYTES,
};

const FLOAT_TOLERANCE: f64 = 1e-12;
const MAX_EXACT_INTEGER: u64 = 1 << 53;
const SCIENTIFIC_METRICS: [&str; 4] =
    ["primary_reads", "mapped_primary_reads", "mapped_primary_fraction", "paired_primary_reads"];

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct MappingSummaryBundle {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_id: String,
    primary_reads: u64,
    mapped_primary_reads: u64,
    mapped_primary_fraction: f64,
    paired_primary_reads: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct MappingSummaryMetrics {
    primary_reads: u64,
    mapped_primary_reads: u64,
    mapped_primary_fraction: f64,
    paired_primary_reads: u64,
    bam_input_bytes: u64,
    bam_index_input_bytes: u64,
    input_bytes: u64,
    output_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct DownstreamProbe {
    schema_version: String,
    stage_id: String,
    input_contract: String,
    complete: bool,
}

#[derive(Debug, Clone)]
pub(super) struct ScientificObservation {
    pub(super) values: BTreeMap<String, BenchmarkScientificValue>,
    pub(super) bam_input_bytes: u64,
    pub(super) bam_index_input_bytes: u64,
    pub(super) input_bytes: u64,
    pub(super) output_bytes: u64,
    pub(super) normalized_bundle_sha256: String,
}

pub(super) fn derive_scientific_observation(
    spec: &BenchmarkExecutionSpec,
    repetition: u32,
) -> Result<ScientificObservation> {
    let result_root = Path::new(&spec.outputs.result_root);
    let prefix = format!("repetitions/repetition-{repetition}");
    let summary_path = result_root.join(format!("{prefix}.mapping-summary.tsv"));
    let summary = read_summary(&summary_path)?;
    let bundle: MappingSummaryBundle =
        read_json(&result_root.join(format!("{prefix}.mapping-summary.json")))?;
    let metrics: MappingSummaryMetrics =
        read_json(&result_root.join(format!("{prefix}.metrics.json")))?;
    let probe: DownstreamProbe = read_json(&result_root.join(format!("{prefix}.report-qc.json")))?;
    validate_bundle(spec, &bundle, &summary)?;
    validate_metrics(&metrics, &summary)?;
    if probe.schema_version != "bijux.benchmark.downstream_probe.v1"
        || probe.stage_id != "benchmark.bam.mapping_summary.compare"
        || probe.input_contract != BUNDLE_SCHEMA
        || !probe.complete
    {
        bail!("BAM mapping-summary downstream probe is not complete");
    }
    let expected_input_bytes = metrics
        .bam_input_bytes
        .checked_add(metrics.bam_index_input_bytes)
        .context("BAM mapping-summary input size overflow")?;
    let bam_input_bytes = input_size(spec, "bam")?;
    let bam_index_input_bytes = input_size(spec, "bam_index")?;
    let raw_suffix = if spec.identity.tool_id == "picard" { "picard.tsv" } else { "flagstat.txt" };
    let output_bytes = [
        result_root.join(format!("{prefix}.{raw_suffix}")),
        summary_path.clone(),
        result_root.join(format!("{prefix}.mapping-summary.json")),
        result_root.join(format!("{prefix}.report-qc.json")),
    ]
    .iter()
    .try_fold(0_u64, |total, path| {
        total
            .checked_add(regular_file_size(path)?)
            .context("BAM mapping-summary output size overflow")
    })?;
    if metrics.bam_input_bytes == 0
        || metrics.bam_index_input_bytes == 0
        || metrics.input_bytes != expected_input_bytes
        || metrics.bam_input_bytes != bam_input_bytes
        || metrics.bam_index_input_bytes != bam_index_input_bytes
        || metrics.output_bytes == 0
        || metrics.output_bytes != output_bytes
    {
        bail!("BAM mapping-summary metrics contain invalid byte counts");
    }
    Ok(ScientificObservation {
        values: summary,
        bam_input_bytes: metrics.bam_input_bytes,
        bam_index_input_bytes: metrics.bam_index_input_bytes,
        input_bytes: metrics.input_bytes,
        output_bytes: metrics.output_bytes,
        normalized_bundle_sha256: bijux_dna_infra::hash_file_sha256(&summary_path)?,
    })
}

fn input_size(spec: &BenchmarkExecutionSpec, id: &str) -> Result<u64> {
    let input = spec
        .inputs
        .iter()
        .find(|input| input.id == id)
        .with_context(|| format!("BAM mapping-summary execution omits {id}"))?;
    regular_file_size(Path::new(&input.path))
}

fn regular_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect BAM mapping-summary evidence {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("BAM mapping-summary evidence is not a regular non-symlink file");
    }
    Ok(metadata.len())
}

fn read_summary(path: &Path) -> Result<BTreeMap<String, BenchmarkScientificValue>> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)
        .with_context(|| format!("read BAM mapping summary {}", path.display()))?;
    let text = std::str::from_utf8(&bytes).context("BAM mapping-summary TSV is not UTF-8")?;
    let mut lines = text.lines();
    if lines.next() != Some("metric\tvalue") {
        bail!("BAM mapping-summary TSV header is invalid");
    }
    let mut values = BTreeMap::new();
    for line in lines {
        let (metric, value) =
            line.split_once('\t').context("BAM mapping-summary row is invalid")?;
        if !SCIENTIFIC_METRICS.contains(&metric) || value.contains('\t') {
            bail!("BAM mapping-summary metric is outside the contract");
        }
        let value = value.parse::<f64>().context("parse BAM mapping-summary metric")?;
        if !value.is_finite()
            || value <= 0.0
            || values.insert(metric.to_string(), number(value)).is_some()
        {
            bail!("BAM mapping-summary metric is invalid or duplicated");
        }
    }
    let observed = values.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = SCIENTIFIC_METRICS.into_iter().collect::<BTreeSet<_>>();
    if observed != expected {
        bail!("BAM mapping-summary TSV lacks exact metric closure");
    }
    Ok(values)
}

fn validate_bundle(
    spec: &BenchmarkExecutionSpec,
    bundle: &MappingSummaryBundle,
    summary: &BTreeMap<String, BenchmarkScientificValue>,
) -> Result<()> {
    if bundle.schema_version != BUNDLE_SCHEMA
        || bundle.stage_id != STAGE_ID
        || bundle.tool_id != spec.identity.tool_id
        || bundle.sample_id != super::super::SAMPLE_ID
        || bundle.primary_reads == 0
        || bundle.mapped_primary_reads == 0
        || bundle.mapped_primary_reads > bundle.primary_reads
        || bundle.paired_primary_reads == 0
        || bundle.paired_primary_reads > bundle.primary_reads
        || !bundle.mapped_primary_fraction.is_finite()
        || bundle.mapped_primary_fraction <= 0.0
        || bundle.mapped_primary_fraction > 1.0
    {
        bail!("BAM mapping-summary bundle identity or values are invalid");
    }
    validate_values(
        summary,
        bundle.primary_reads,
        bundle.mapped_primary_reads,
        bundle.mapped_primary_fraction,
        bundle.paired_primary_reads,
        "bundle",
    )
}

fn validate_metrics(
    metrics: &MappingSummaryMetrics,
    summary: &BTreeMap<String, BenchmarkScientificValue>,
) -> Result<()> {
    validate_values(
        summary,
        metrics.primary_reads,
        metrics.mapped_primary_reads,
        metrics.mapped_primary_fraction,
        metrics.paired_primary_reads,
        "metrics",
    )
}

fn validate_values(
    summary: &BTreeMap<String, BenchmarkScientificValue>,
    primary_reads: u64,
    mapped_primary_reads: u64,
    mapped_primary_fraction: f64,
    paired_primary_reads: u64,
    source: &str,
) -> Result<()> {
    let observed = |metric: &str| match summary.get(metric) {
        Some(BenchmarkScientificValue::Number(value)) => Ok(*value),
        _ => bail!("BAM mapping-summary TSV omits numeric {metric}"),
    };
    let fraction = mapped_primary_reads as f64 / primary_reads as f64;
    if primary_reads == 0
        || mapped_primary_reads == 0
        || paired_primary_reads == 0
        || primary_reads > MAX_EXACT_INTEGER
        || mapped_primary_reads > MAX_EXACT_INTEGER
        || paired_primary_reads > MAX_EXACT_INTEGER
        || mapped_primary_reads > primary_reads
        || paired_primary_reads > primary_reads
        || !mapped_primary_fraction.is_finite()
        || (mapped_primary_fraction - fraction).abs() > FLOAT_TOLERANCE
        || observed("primary_reads")? != primary_reads as f64
        || observed("mapped_primary_reads")? != mapped_primary_reads as f64
        || (observed("mapped_primary_fraction")? - mapped_primary_fraction).abs() > FLOAT_TOLERANCE
        || observed("paired_primary_reads")? != paired_primary_reads as f64
    {
        bail!("BAM mapping-summary {source} disagrees with normalized TSV");
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

fn number(value: f64) -> BenchmarkScientificValue {
    BenchmarkScientificValue::Number(value)
}

#[cfg(test)]
mod tests {
    use super::read_summary;

    #[test]
    fn normalized_summary_requires_exact_metric_closure() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("bam-mapping-summary-report")?;
        let path = temporary.path().join("mapping-summary.tsv");
        std::fs::write(
            &path,
            b"metric\tvalue\nprimary_reads\t100\nmapped_primary_reads\t80\nmapped_primary_fraction\t0.800000000000\npaired_primary_reads\t90\n",
        )?;
        let summary = read_summary(&path)?;
        assert_eq!(summary.len(), 4);
        std::fs::write(&path, b"metric\tvalue\nprimary_reads\t100\n")?;
        assert!(read_summary(&path).is_err());
        Ok(())
    }
}
