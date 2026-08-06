//! Owner: bijux-dna-bench
//! Scientific observations derived from validated FASTQ profile outputs.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::normalized_result::fastq::{
    BenchmarkFastqProfileMetrics as ProfileMetrics, BenchmarkFastqProfileProbe as ProfileProbe,
    BenchmarkFastqProfileResult as ProfileBundle, BenchmarkFastqProfileResultRow as ProfileRow,
    FASTQ_PROFILE_READS_METRICS_SCHEMA_V1 as METRICS_SCHEMA,
    FASTQ_PROFILE_READS_PROBE_SCHEMA_V1 as PROBE_SCHEMA,
    FASTQ_PROFILE_READS_RESULT_SCHEMA_V1 as BUNDLE_SCHEMA,
    FASTQ_PROFILE_READS_STAGE_ID as STAGE_ID,
};
use bijux_dna_bench_model::{
    BenchmarkExecutionSpec, BenchmarkMetricDistribution, BenchmarkScientificValue,
};

use super::{read_json, MAX_REPORT_INPUT_BYTES};
use crate::workflow::stage_report::{distribution, ResourceObservation};

pub(super) struct ScientificObservation {
    pub(super) values: BTreeMap<String, BenchmarkScientificValue>,
    pub(super) input_bytes: u64,
    pub(super) output_bytes: u64,
    pub(super) profile_sha256: String,
}

pub(super) fn derive_scientific_observation(
    spec: &BenchmarkExecutionSpec,
    repetition: u32,
) -> Result<ScientificObservation> {
    let root = Path::new(&spec.outputs.result_root);
    let prefix = format!("repetitions/repetition-{repetition}");
    let metrics: ProfileMetrics = read_json(&root.join(format!("{prefix}.metrics.json")))?;
    let probe: ProfileProbe = read_json(&root.join(format!("{prefix}.report-qc.json")))?;
    let bundle: ProfileBundle = read_json(&root.join(format!("{prefix}.read-profile.json")))?;
    let profile_path = root.join(format!("{prefix}.read-profile.tsv"));
    let rows = read_profile(&profile_path)?;
    if bundle.schema_version != BUNDLE_SCHEMA
        || bundle.stage_id != STAGE_ID
        || bundle.tool_id != spec.identity.tool_id
        || bundle.results != rows
    {
        bail!("FASTQ profile bundle differs from normalized TSV evidence");
    }
    let aggregate = aggregate(&rows)?;
    if metrics.schema_version != METRICS_SCHEMA
        || metrics.admission_mismatch_count != 0
        || metrics.input_bytes == 0
        || metrics.output_bytes == 0
        || aggregate != metric_tuple(&metrics)
    {
        bail!("FASTQ profile metrics are not admissible");
    }
    if probe.schema_version != PROBE_SCHEMA
        || probe.consumer_stage_id != "benchmark.fastq.profile_reads.compare"
        || probe.input_contract != BUNDLE_SCHEMA
        || aggregate != probe_tuple(&probe)
    {
        bail!("FASTQ profile downstream probe differs from normalized evidence");
    }
    Ok(ScientificObservation {
        values: BTreeMap::from([
            ("file_count".to_string(), number(metrics.file_count)),
            ("read_count".to_string(), number(metrics.read_count)),
            ("base_count".to_string(), number(metrics.base_count)),
            ("minimum_read_length".to_string(), number(metrics.minimum_read_length)),
            ("maximum_read_length".to_string(), number(metrics.maximum_read_length)),
        ]),
        input_bytes: metrics.input_bytes,
        output_bytes: metrics.output_bytes,
        profile_sha256: bijux_dna_infra::hash_file_sha256(&profile_path)?,
    })
}

fn read_profile(path: &Path) -> Result<Vec<ProfileRow>> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)?;
    let text = std::str::from_utf8(&bytes).context("FASTQ profile TSV is not UTF-8")?;
    let mut lines = text.lines();
    if lines.next()
        != Some("sample_id\tinput_layout\tfile_count\tread_count\tbase_count\tminimum_read_length\tmaximum_read_length")
    {
        bail!("FASTQ profile TSV header is invalid");
    }
    lines
        .map(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            if fields.len() != 7 {
                bail!("FASTQ profile TSV row has invalid field count");
            }
            Ok(ProfileRow {
                sample_id: fields[0].to_string(),
                input_layout: fields[1].to_string(),
                file_count: fields[2].parse()?,
                read_count: fields[3].parse()?,
                base_count: fields[4].parse()?,
                minimum_read_length: fields[5].parse()?,
                maximum_read_length: fields[6].parse()?,
            })
        })
        .collect()
}

type Aggregate = (u64, u64, u64, u64, u64, u64);

fn aggregate(rows: &[ProfileRow]) -> Result<Aggregate> {
    if rows.is_empty() {
        bail!("FASTQ profile contains no samples");
    }
    let mut samples = BTreeSet::new();
    let mut totals = (rows.len() as u64, 0_u64, 0_u64, 0_u64, u64::MAX, 0_u64);
    for row in rows {
        if row.sample_id.trim().is_empty()
            || !samples.insert(row.sample_id.as_str())
            || !matches!(row.input_layout.as_str(), "single_end" | "paired_end")
            || row.file_count == 0
            || row.read_count == 0
            || row.base_count == 0
            || row.minimum_read_length == 0
            || row.maximum_read_length < row.minimum_read_length
        {
            bail!("FASTQ profile contains an invalid sample row");
        }
        totals.1 = totals.1.checked_add(row.file_count).context("FASTQ file count overflow")?;
        totals.2 = totals.2.checked_add(row.read_count).context("FASTQ read count overflow")?;
        totals.3 = totals.3.checked_add(row.base_count).context("FASTQ base count overflow")?;
        totals.4 = totals.4.min(row.minimum_read_length);
        totals.5 = totals.5.max(row.maximum_read_length);
    }
    Ok(totals)
}

fn metric_tuple(value: &ProfileMetrics) -> Aggregate {
    (
        value.sample_count,
        value.file_count,
        value.read_count,
        value.base_count,
        value.minimum_read_length,
        value.maximum_read_length,
    )
}

fn probe_tuple(value: &ProfileProbe) -> Aggregate {
    (
        value.sample_count,
        value.file_count,
        value.read_count,
        value.base_count,
        value.minimum_read_length,
        value.maximum_read_length,
    )
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
    ]))
}

fn number(value: u64) -> BenchmarkScientificValue {
    BenchmarkScientificValue::Number(value as f64)
}
