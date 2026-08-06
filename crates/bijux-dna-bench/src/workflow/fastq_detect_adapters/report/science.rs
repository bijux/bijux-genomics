//! Owner: bijux-dna-bench
//! Scientific observations derived from normalized FASTQ adapter-inspection outputs.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::normalized_result::fastq::{
    BenchmarkFastqAdapterMetrics as Metrics, BenchmarkFastqAdapterProbe as Probe,
    BenchmarkFastqAdapterResult as Bundle, BenchmarkFastqAdapterResultRow as BundleRow,
    FASTQ_DETECT_ADAPTERS_METRICS_SCHEMA_V1 as METRICS_SCHEMA,
    FASTQ_DETECT_ADAPTERS_PROBE_SCHEMA_V1 as PROBE_SCHEMA,
    FASTQ_DETECT_ADAPTERS_RESULT_SCHEMA_V1 as BUNDLE_SCHEMA,
    FASTQ_DETECT_ADAPTERS_STAGE_ID as STAGE_ID,
};
use bijux_dna_bench_model::{
    BenchmarkExecutionSpec, BenchmarkMetricDistribution, BenchmarkScientificValue,
};

use super::{read_json, MAX_REPORT_INPUT_BYTES};
use crate::workflow::stage_report::{distribution, ResourceObservation};

const FLOAT_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq)]
struct SummaryRow {
    sample_id: String,
    input_layout: String,
    file_count: u64,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    mean_q: f64,
    candidate_adapter_count: u64,
    detected_adapter_ids: Vec<String>,
    detection_confidence: Option<f64>,
    detection_threshold: f64,
    adapter_trimmed_fraction: f64,
    adapter_content_max: f64,
    adapter_content_mean: f64,
    duplication_rate: f64,
    n_rate: f64,
    kmer_warning_count: u64,
    overrepresented_sequence_count: u64,
}

pub(super) struct ScientificObservation {
    pub(super) values: BTreeMap<String, BenchmarkScientificValue>,
    pub(super) input_bytes: u64,
    pub(super) output_bytes: u64,
    pub(super) summary_sha256: String,
}

#[derive(Debug, Clone, PartialEq)]
struct Aggregate {
    sample_count: u64,
    file_count: u64,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    mean_q_mean: f64,
    candidate_adapter_count_total: u64,
    samples_with_detected_adapters: u64,
    max_detection_confidence: f64,
    max_adapter_content_max: f64,
    adapter_content_mean_mean: f64,
    duplication_rate_mean: f64,
    n_rate_mean: f64,
    kmer_warning_count: u64,
    overrepresented_sequence_count: u64,
}

pub(super) fn derive_scientific_observation(
    spec: &BenchmarkExecutionSpec,
    repetition: u32,
) -> Result<ScientificObservation> {
    let root = Path::new(&spec.outputs.result_root);
    let prefix = format!("repetitions/repetition-{repetition}");
    let metrics: Metrics = read_json(&root.join(format!("{prefix}.metrics.json")))?;
    let bundle: Bundle = read_json(&root.join(format!("{prefix}.adapter-bundle.json")))?;
    let probe: Probe = read_json(&root.join(format!("{prefix}.report-qc.json")))?;
    let summary_path = root.join(format!("{prefix}.adapter-summary.tsv"));
    let observed = aggregate(&read_summary(&summary_path)?)?;
    if metrics.schema_version != METRICS_SCHEMA
        || metrics.input_bytes == 0
        || metrics.output_bytes == 0
        || !aggregate_matches(&observed, &aggregate_from_metrics(&metrics))
    {
        bail!("FASTQ detect-adapters metrics are not admissible");
    }
    if bundle.schema_version != BUNDLE_SCHEMA
        || bundle.stage_id != STAGE_ID
        || bundle.tool_id != spec.identity.tool_id
        || !bundle_matches_summary(&bundle.results, &read_summary(&summary_path)?)?
    {
        bail!("FASTQ detect-adapters bundle differs from normalized evidence");
    }
    if probe.schema_version != PROBE_SCHEMA
        || probe.consumer_stage_id != "benchmark.fastq.detect_adapters.compare"
        || probe.input_contract != BUNDLE_SCHEMA
        || probe.sample_count != observed.sample_count
        || probe.file_count != observed.file_count
        || probe.reads_in != observed.reads_in
        || probe.candidate_adapter_count_total != observed.candidate_adapter_count_total
        || probe.samples_with_detected_adapters != observed.samples_with_detected_adapters
        || (probe.max_detection_confidence - observed.max_detection_confidence).abs()
            > FLOAT_TOLERANCE
    {
        bail!("FASTQ detect-adapters downstream probe differs from normalized evidence");
    }
    if observed.sample_count != 20
        || observed.file_count != 30
        || observed.reads_in == 0
        || observed.bases_in == 0
        || observed.mean_q_mean <= 0.0
    {
        bail!("FASTQ detect-adapters scientific closure did not pass");
    }
    Ok(ScientificObservation {
        values: BTreeMap::from([
            ("sample_count".to_string(), number(observed.sample_count)),
            ("file_count".to_string(), number(observed.file_count)),
            ("reads_in".to_string(), number(observed.reads_in)),
            (
                "samples_with_detected_adapters".to_string(),
                number(observed.samples_with_detected_adapters),
            ),
            (
                "max_detection_confidence".to_string(),
                BenchmarkScientificValue::Number(observed.max_detection_confidence),
            ),
            (
                "max_adapter_content_max".to_string(),
                BenchmarkScientificValue::Number(observed.max_adapter_content_max),
            ),
            (
                "adapter_content_mean_mean".to_string(),
                BenchmarkScientificValue::Number(observed.adapter_content_mean_mean),
            ),
            (
                "duplication_rate_mean".to_string(),
                BenchmarkScientificValue::Number(observed.duplication_rate_mean),
            ),
            (
                "overrepresented_sequence_count".to_string(),
                number(observed.overrepresented_sequence_count),
            ),
        ]),
        input_bytes: metrics.input_bytes,
        output_bytes: metrics.output_bytes,
        summary_sha256: bijux_dna_infra::hash_file_sha256(&summary_path)?,
    })
}

fn read_summary(path: &Path) -> Result<Vec<SummaryRow>> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)?;
    let text = std::str::from_utf8(&bytes).context("FASTQ detect-adapters TSV is not UTF-8")?;
    let mut lines = text.lines();
    if lines.next()
        != Some("sample_id\tinput_layout\tfile_count\treads_in\treads_out\tbases_in\tbases_out\tmean_q\tcandidate_adapter_count\tdetected_adapter_ids\tdetection_confidence\tdetection_threshold\tadapter_trimmed_fraction\tadapter_content_max\tadapter_content_mean\tduplication_rate\tn_rate\tkmer_warning_count\toverrepresented_sequence_count")
    {
        bail!("FASTQ detect-adapters TSV header is invalid");
    }
    lines
        .map(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            if fields.len() != 19 {
                bail!("FASTQ detect-adapters TSV row has invalid field count");
            }
            Ok(SummaryRow {
                sample_id: fields[0].to_string(),
                input_layout: fields[1].to_string(),
                file_count: fields[2].parse()?,
                reads_in: fields[3].parse()?,
                reads_out: fields[4].parse()?,
                bases_in: fields[5].parse()?,
                bases_out: fields[6].parse()?,
                mean_q: fields[7].parse()?,
                candidate_adapter_count: fields[8].parse()?,
                detected_adapter_ids: if fields[9].is_empty() {
                    Vec::new()
                } else {
                    fields[9].split(',').map(str::to_string).collect()
                },
                detection_confidence: if fields[10].is_empty() {
                    None
                } else {
                    Some(fields[10].parse()?)
                },
                detection_threshold: fields[11].parse()?,
                adapter_trimmed_fraction: fields[12].parse()?,
                adapter_content_max: fields[13].parse()?,
                adapter_content_mean: fields[14].parse()?,
                duplication_rate: fields[15].parse()?,
                n_rate: fields[16].parse()?,
                kmer_warning_count: fields[17].parse()?,
                overrepresented_sequence_count: fields[18].parse()?,
            })
        })
        .collect()
}

fn aggregate(rows: &[SummaryRow]) -> Result<Aggregate> {
    let mut samples = BTreeSet::new();
    let mut aggregate = Aggregate {
        sample_count: 0,
        file_count: 0,
        reads_in: 0,
        reads_out: 0,
        bases_in: 0,
        bases_out: 0,
        mean_q_mean: 0.0,
        candidate_adapter_count_total: 0,
        samples_with_detected_adapters: 0,
        max_detection_confidence: 0.0,
        max_adapter_content_max: 0.0,
        adapter_content_mean_mean: 0.0,
        duplication_rate_mean: 0.0,
        n_rate_mean: 0.0,
        kmer_warning_count: 0,
        overrepresented_sequence_count: 0,
    };
    for row in rows {
        let valid_layout = match row.input_layout.as_str() {
            "single_end" => row.file_count == 1,
            "paired_end" => row.file_count == 2,
            _ => false,
        };
        if row.sample_id.is_empty()
            || !samples.insert(row.sample_id.as_str())
            || !valid_layout
            || row.reads_in == 0
            || row.reads_in != row.reads_out
            || row.bases_in == 0
            || row.bases_in != row.bases_out
            || row.mean_q <= 0.0
            || row.detection_threshold <= 0.0
            || row.candidate_adapter_count as usize != row.detected_adapter_ids.len()
            || (row.candidate_adapter_count == 0) != row.detection_confidence.is_none()
        {
            bail!("FASTQ detect-adapters summary contains an invalid sample row");
        }
        aggregate.sample_count += 1;
        aggregate.file_count = aggregate
            .file_count
            .checked_add(row.file_count)
            .context("FASTQ file count overflow")?;
        aggregate.reads_in =
            aggregate.reads_in.checked_add(row.reads_in).context("FASTQ read count overflow")?;
        aggregate.reads_out = aggregate
            .reads_out
            .checked_add(row.reads_out)
            .context("FASTQ output read count overflow")?;
        aggregate.bases_in =
            aggregate.bases_in.checked_add(row.bases_in).context("FASTQ base count overflow")?;
        aggregate.bases_out = aggregate
            .bases_out
            .checked_add(row.bases_out)
            .context("FASTQ output base count overflow")?;
        aggregate.mean_q_mean += row.mean_q;
        aggregate.candidate_adapter_count_total = aggregate
            .candidate_adapter_count_total
            .checked_add(row.candidate_adapter_count)
            .context("FASTQ candidate adapter count overflow")?;
        aggregate.samples_with_detected_adapters += u64::from(row.candidate_adapter_count > 0);
        if let Some(confidence) = row.detection_confidence {
            if confidence > aggregate.max_detection_confidence {
                aggregate.max_detection_confidence = confidence;
            }
        }
        if row.adapter_content_max > aggregate.max_adapter_content_max {
            aggregate.max_adapter_content_max = row.adapter_content_max;
        }
        aggregate.adapter_content_mean_mean += row.adapter_content_mean;
        aggregate.duplication_rate_mean += row.duplication_rate;
        aggregate.n_rate_mean += row.n_rate;
        aggregate.kmer_warning_count = aggregate
            .kmer_warning_count
            .checked_add(row.kmer_warning_count)
            .context("FASTQ kmer warning count overflow")?;
        aggregate.overrepresented_sequence_count = aggregate
            .overrepresented_sequence_count
            .checked_add(row.overrepresented_sequence_count)
            .context("FASTQ overrepresented count overflow")?;
    }
    if aggregate.sample_count == 0 {
        bail!("FASTQ detect-adapters summary contains no samples");
    }
    let sample_count = aggregate.sample_count as f64;
    aggregate.mean_q_mean /= sample_count;
    aggregate.adapter_content_mean_mean /= sample_count;
    aggregate.duplication_rate_mean /= sample_count;
    aggregate.n_rate_mean /= sample_count;
    Ok(aggregate)
}

fn bundle_matches_summary(bundle: &[BundleRow], summary: &[SummaryRow]) -> Result<bool> {
    if bundle.len() != summary.len() {
        return Ok(false);
    }
    let by_sample =
        summary.iter().map(|row| (row.sample_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    for row in bundle {
        let summary = by_sample
            .get(row.sample_id.as_str())
            .with_context(|| format!("bundle sample {} is absent from summary", row.sample_id))?;
        if row.input_layout != summary.input_layout
            || row.file_count != summary.file_count
            || row.reads_in != summary.reads_in
            || row.reads_out != summary.reads_out
            || row.bases_in != summary.bases_in
            || row.bases_out != summary.bases_out
            || (row.mean_q - summary.mean_q).abs() > FLOAT_TOLERANCE
            || row.candidate_adapter_count != summary.candidate_adapter_count
            || row.detected_adapter_ids != summary.detected_adapter_ids
            || !optional_equal(row.detection_confidence, summary.detection_confidence)
            || (row.detection_threshold - summary.detection_threshold).abs() > FLOAT_TOLERANCE
            || (row.adapter_trimmed_fraction - summary.adapter_trimmed_fraction).abs()
                > FLOAT_TOLERANCE
            || (row.adapter_content_max - summary.adapter_content_max).abs() > FLOAT_TOLERANCE
            || (row.adapter_content_mean - summary.adapter_content_mean).abs() > FLOAT_TOLERANCE
            || (row.duplication_rate - summary.duplication_rate).abs() > FLOAT_TOLERANCE
            || (row.n_rate - summary.n_rate).abs() > FLOAT_TOLERANCE
            || row.kmer_warning_count != summary.kmer_warning_count
            || row.overrepresented_sequence_count != summary.overrepresented_sequence_count
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn aggregate_from_metrics(metrics: &Metrics) -> Aggregate {
    Aggregate {
        sample_count: metrics.sample_count,
        file_count: metrics.file_count,
        reads_in: metrics.reads_in,
        reads_out: metrics.reads_out,
        bases_in: metrics.bases_in,
        bases_out: metrics.bases_out,
        mean_q_mean: metrics.mean_q_mean,
        candidate_adapter_count_total: metrics.candidate_adapter_count_total,
        samples_with_detected_adapters: metrics.samples_with_detected_adapters,
        max_detection_confidence: metrics.max_detection_confidence,
        max_adapter_content_max: metrics.max_adapter_content_max,
        adapter_content_mean_mean: metrics.adapter_content_mean_mean,
        duplication_rate_mean: metrics.duplication_rate_mean,
        n_rate_mean: metrics.n_rate_mean,
        kmer_warning_count: metrics.kmer_warning_count,
        overrepresented_sequence_count: metrics.overrepresented_sequence_count,
    }
}

fn aggregate_matches(left: &Aggregate, right: &Aggregate) -> bool {
    left.sample_count == right.sample_count
        && left.file_count == right.file_count
        && left.reads_in == right.reads_in
        && left.reads_out == right.reads_out
        && left.bases_in == right.bases_in
        && left.bases_out == right.bases_out
        && left.candidate_adapter_count_total == right.candidate_adapter_count_total
        && left.samples_with_detected_adapters == right.samples_with_detected_adapters
        && left.kmer_warning_count == right.kmer_warning_count
        && left.overrepresented_sequence_count == right.overrepresented_sequence_count
        && (left.mean_q_mean - right.mean_q_mean).abs() <= FLOAT_TOLERANCE
        && (left.max_detection_confidence - right.max_detection_confidence).abs() <= FLOAT_TOLERANCE
        && (left.max_adapter_content_max - right.max_adapter_content_max).abs() <= FLOAT_TOLERANCE
        && (left.adapter_content_mean_mean - right.adapter_content_mean_mean).abs()
            <= FLOAT_TOLERANCE
        && (left.duplication_rate_mean - right.duplication_rate_mean).abs() <= FLOAT_TOLERANCE
        && (left.n_rate_mean - right.n_rate_mean).abs() <= FLOAT_TOLERANCE
}

fn optional_equal(left: Option<f64>, right: Option<f64>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => (left - right).abs() <= FLOAT_TOLERANCE,
        (None, None) => true,
        _ => false,
    }
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
            distribution(
                &science
                    .iter()
                    .map(|observation| observation.output_bytes as f64)
                    .collect::<Vec<_>>(),
            )?,
        ),
        (
            "throughput_input_bytes_per_second".to_string(),
            distribution(&wall.iter().map(|wall| input / wall).collect::<Vec<_>>())?,
        ),
    ]))
}

fn number(value: u64) -> BenchmarkScientificValue {
    BenchmarkScientificValue::Number(value as f64)
}
