//! Owner: bijux-dna-bench
//! Scientific observations derived from validated FASTQ validation outputs.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkExecutionSpec, BenchmarkMetricDistribution, BenchmarkScientificValue,
};
use serde::Deserialize;

use super::{
    read_json, BUNDLE_SCHEMA, MAX_REPORT_INPUT_BYTES, METRICS_SCHEMA, PROBE_SCHEMA, STAGE_ID,
};
use crate::workflow::stage_report::{distribution, ResourceObservation};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationRow {
    sample_id: String,
    input_layout: String,
    file_count: u64,
    expected_read_count: u64,
    validated_read_count: u64,
    strict_pass: bool,
    pair_sync_checked: bool,
    pair_sync_pass: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationAggregate {
    schema_version: String,
    #[serde(default)]
    stage_id: Option<String>,
    #[serde(default)]
    tool_id: Option<String>,
    sample_count: u64,
    file_count: u64,
    expected_read_count: u64,
    validated_read_count: u64,
    strict_pass_count: u64,
    pair_sync_checked_count: u64,
    pair_sync_failure_count: u64,
    #[serde(default)]
    input_bytes: Option<u64>,
    #[serde(default)]
    output_bytes: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationProbe {
    schema_version: String,
    consumer_stage_id: String,
    input_contract: String,
    sample_count: u64,
    file_count: u64,
    validated_read_count: u64,
    strict_pass_count: u64,
    pair_sync_failure_count: u64,
}

pub(super) struct ScientificObservation {
    pub(super) values: BTreeMap<String, BenchmarkScientificValue>,
    pub(super) input_bytes: u64,
    pub(super) output_bytes: u64,
    pub(super) summary_sha256: String,
}

pub(super) fn derive_scientific_observation(
    spec: &BenchmarkExecutionSpec,
    repetition: u32,
) -> Result<ScientificObservation> {
    let root = Path::new(&spec.outputs.result_root);
    let prefix = format!("repetitions/repetition-{repetition}");
    let metrics: ValidationAggregate = read_json(&root.join(format!("{prefix}.metrics.json")))?;
    let bundle: ValidationAggregate =
        read_json(&root.join(format!("{prefix}.validation-bundle.json")))?;
    let probe: ValidationProbe = read_json(&root.join(format!("{prefix}.report-qc.json")))?;
    let summary_path = root.join(format!("{prefix}.validation-summary.tsv"));
    let observed = aggregate(&read_summary(&summary_path)?)?;
    let metric_values = aggregate_tuple(&metrics);
    if metrics.schema_version != METRICS_SCHEMA
        || metrics.stage_id.is_some()
        || metrics.tool_id.is_some()
        || metrics.input_bytes.is_none_or(|value| value == 0)
        || metrics.output_bytes.is_none_or(|value| value == 0)
        || observed != metric_values
    {
        bail!("FASTQ validation metrics are not admissible");
    }
    if bundle.schema_version != BUNDLE_SCHEMA
        || bundle.stage_id.as_deref() != Some(STAGE_ID)
        || bundle.tool_id.as_deref() != Some(&spec.identity.tool_id)
        || bundle.input_bytes.is_some()
        || bundle.output_bytes.is_some()
        || observed != aggregate_tuple(&bundle)
    {
        bail!("FASTQ validation bundle differs from normalized evidence");
    }
    if probe.schema_version != PROBE_SCHEMA
        || probe.consumer_stage_id != "benchmark.fastq.validate_reads.compare"
        || probe.input_contract != BUNDLE_SCHEMA
        || probe_tuple(&probe) != (observed.0, observed.1, observed.3, observed.4, observed.6)
    {
        bail!("FASTQ validation downstream probe differs from normalized evidence");
    }
    if observed.0 != 20
        || observed.1 != 30
        || observed.2 != observed.3
        || observed.4 != 20
        || observed.5 == 0
        || observed.6 != 0
    {
        bail!("FASTQ validation scientific closure did not pass");
    }
    Ok(ScientificObservation {
        values: BTreeMap::from([
            ("sample_count".to_string(), number(observed.0)),
            ("file_count".to_string(), number(observed.1)),
            ("validated_read_count".to_string(), number(observed.3)),
            ("strict_pass_count".to_string(), number(observed.4)),
            ("pair_sync_checked_count".to_string(), number(observed.5)),
            ("pair_sync_failure_count".to_string(), number(observed.6)),
        ]),
        input_bytes: metrics.input_bytes.unwrap_or_default(),
        output_bytes: metrics.output_bytes.unwrap_or_default(),
        summary_sha256: bijux_dna_infra::hash_file_sha256(&summary_path)?,
    })
}

fn read_summary(path: &Path) -> Result<Vec<ValidationRow>> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)?;
    let text = std::str::from_utf8(&bytes).context("FASTQ validation TSV is not UTF-8")?;
    let mut lines = text.lines();
    if lines.next()
        != Some("sample_id\tinput_layout\tfile_count\texpected_read_count\tvalidated_read_count\tstrict_pass\tpair_sync_checked\tpair_sync_pass")
    {
        bail!("FASTQ validation TSV header is invalid");
    }
    lines
        .map(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            if fields.len() != 8 {
                bail!("FASTQ validation TSV row has invalid field count");
            }
            Ok(ValidationRow {
                sample_id: fields[0].to_string(),
                input_layout: fields[1].to_string(),
                file_count: fields[2].parse()?,
                expected_read_count: fields[3].parse()?,
                validated_read_count: fields[4].parse()?,
                strict_pass: parse_bool(fields[5])?,
                pair_sync_checked: parse_bool(fields[6])?,
                pair_sync_pass: parse_optional_bool(fields[7])?,
            })
        })
        .collect()
}

type Aggregate = (u64, u64, u64, u64, u64, u64, u64);

fn aggregate(rows: &[ValidationRow]) -> Result<Aggregate> {
    let mut samples = BTreeSet::new();
    let mut totals = (0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64, 0_u64);
    for row in rows {
        let valid_layout = match row.input_layout.as_str() {
            "single_end" => {
                row.file_count == 1 && !row.pair_sync_checked && row.pair_sync_pass.is_none()
            }
            "paired_end" => {
                row.file_count == 2 && row.pair_sync_checked && row.pair_sync_pass == Some(true)
            }
            _ => false,
        };
        if row.sample_id.is_empty()
            || !samples.insert(row.sample_id.as_str())
            || !valid_layout
            || row.expected_read_count == 0
            || row.expected_read_count != row.validated_read_count
            || !row.strict_pass
        {
            bail!("FASTQ validation summary contains an invalid sample row");
        }
        totals.0 += 1;
        totals.1 = totals.1.checked_add(row.file_count).context("FASTQ file count overflow")?;
        totals.2 = totals
            .2
            .checked_add(row.expected_read_count)
            .context("FASTQ expected read count overflow")?;
        totals.3 = totals
            .3
            .checked_add(row.validated_read_count)
            .context("FASTQ validated read count overflow")?;
        totals.4 += u64::from(row.strict_pass);
        totals.5 += u64::from(row.pair_sync_checked);
        totals.6 += u64::from(row.pair_sync_checked && row.pair_sync_pass != Some(true));
    }
    Ok(totals)
}

fn aggregate_tuple(value: &ValidationAggregate) -> Aggregate {
    (
        value.sample_count,
        value.file_count,
        value.expected_read_count,
        value.validated_read_count,
        value.strict_pass_count,
        value.pair_sync_checked_count,
        value.pair_sync_failure_count,
    )
}

fn probe_tuple(value: &ValidationProbe) -> (u64, u64, u64, u64, u64) {
    (
        value.sample_count,
        value.file_count,
        value.validated_read_count,
        value.strict_pass_count,
        value.pair_sync_failure_count,
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

fn parse_bool(value: &str) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail!("FASTQ validation boolean is invalid"),
    }
}

fn parse_optional_bool(value: &str) -> Result<Option<bool>> {
    match value {
        "null" => Ok(None),
        _ => parse_bool(value).map(Some),
    }
}

fn number(value: u64) -> BenchmarkScientificValue {
    BenchmarkScientificValue::Number(value as f64)
}

#[cfg(test)]
mod tests {
    use super::{aggregate, ValidationRow};

    #[test]
    fn validation_summary_closes_single_and_paired_layouts() -> anyhow::Result<()> {
        let rows = vec![
            ValidationRow {
                sample_id: "single".to_string(),
                input_layout: "single_end".to_string(),
                file_count: 1,
                expected_read_count: 11,
                validated_read_count: 11,
                strict_pass: true,
                pair_sync_checked: false,
                pair_sync_pass: None,
            },
            ValidationRow {
                sample_id: "paired".to_string(),
                input_layout: "paired_end".to_string(),
                file_count: 2,
                expected_read_count: 22,
                validated_read_count: 22,
                strict_pass: true,
                pair_sync_checked: true,
                pair_sync_pass: Some(true),
            },
        ];

        assert_eq!(aggregate(&rows)?, (2, 3, 33, 33, 2, 1, 0));
        Ok(())
    }

    #[test]
    fn validation_summary_rejects_paired_layout_without_sync_closure() {
        let rows = [ValidationRow {
            sample_id: "paired".to_string(),
            input_layout: "paired_end".to_string(),
            file_count: 2,
            expected_read_count: 22,
            validated_read_count: 22,
            strict_pass: true,
            pair_sync_checked: true,
            pair_sync_pass: Some(false),
        }];

        assert!(aggregate(&rows).is_err());
    }
}
