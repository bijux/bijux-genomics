//! Owner: bijux-dna-bench
//! Typed scientific evidence validation for BAM duplication-metrics reports.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Result};
use bijux_dna_bench_model::{BenchmarkExecutionSpec, BenchmarkScientificValue};
use serde::Deserialize;

use super::super::super::bam_observation::{
    numeric_value, read_numeric_summary, validate_byte_counts, ScientificObservation,
};
use crate::workflow::stage_report::read_json;

const BUNDLE_SCHEMA: &str = "bijux.benchmark.bam.duplication_metrics.bundle.v1";
const STAGE_ID: &str = "bam.duplication_metrics";
const PROBE_STAGE_ID: &str = "benchmark.bam.duplication_metrics.compare";
const FLOAT_TOLERANCE: f64 = 1e-12;
const MAX_EXACT_INTEGER: u64 = 1 << 53;
const METRICS: [&str; 3] = ["examined_reads", "duplicate_reads", "duplication_fraction"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DuplicationValues {
    examined_reads: u64,
    duplicate_reads: u64,
    duplication_fraction: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DuplicationBundle {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_id: String,
    #[serde(flatten)]
    values: DuplicationValues,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DuplicationMetrics {
    #[serde(flatten)]
    values: DuplicationValues,
    bam_input_bytes: u64,
    bam_index_input_bytes: u64,
    input_bytes: u64,
    output_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DownstreamProbe {
    schema_version: String,
    stage_id: String,
    input_contract: String,
    complete: bool,
}

pub(super) fn derive_scientific_observation(
    spec: &BenchmarkExecutionSpec,
    repetition: u32,
) -> Result<ScientificObservation> {
    let root = Path::new(&spec.outputs.result_root);
    let prefix = format!("repetitions/repetition-{repetition}");
    let summary_path = root.join(format!("{prefix}.duplication-summary.tsv"));
    let summary = read_numeric_summary(&summary_path, &METRICS)?;
    let bundle: DuplicationBundle =
        read_json(&root.join(format!("{prefix}.duplication-summary.json")))?;
    let metrics: DuplicationMetrics = read_json(&root.join(format!("{prefix}.metrics.json")))?;
    let probe: DownstreamProbe = read_json(&root.join(format!("{prefix}.report-qc.json")))?;

    if bundle.schema_version != BUNDLE_SCHEMA
        || bundle.stage_id != STAGE_ID
        || bundle.tool_id != spec.identity.tool_id
        || bundle.sample_id != super::super::SAMPLE_ID
    {
        bail!("BAM duplication-metrics bundle identity is invalid");
    }
    validate_values(&summary, &bundle.values, "bundle")?;
    validate_values(&summary, &metrics.values, "metrics")?;
    if probe.schema_version != "bijux.benchmark.downstream_probe.v1"
        || probe.stage_id != PROBE_STAGE_ID
        || probe.input_contract != BUNDLE_SCHEMA
        || !probe.complete
    {
        bail!("BAM duplication-metrics downstream probe is not complete");
    }
    let raw_suffix = match spec.identity.tool_id.as_str() {
        "picard" => "picard.metrics.txt",
        "samtools" => "samtools.stats.txt",
        _ => bail!("BAM duplication-metrics report encountered an undeclared tool"),
    };
    let raw = root.join(format!("{prefix}.{raw_suffix}"));
    let summary_json = root.join(format!("{prefix}.duplication-summary.json"));
    let probe_json = root.join(format!("{prefix}.report-qc.json"));
    validate_byte_counts(
        spec,
        metrics.bam_input_bytes,
        metrics.bam_index_input_bytes,
        metrics.input_bytes,
        metrics.output_bytes,
        &[&raw, &summary_path, &summary_json, &probe_json],
    )?;
    Ok(ScientificObservation {
        values: summary,
        bam_input_bytes: metrics.bam_input_bytes,
        bam_index_input_bytes: metrics.bam_index_input_bytes,
        input_bytes: metrics.input_bytes,
        output_bytes: metrics.output_bytes,
        normalized_bundle_sha256: bijux_dna_infra::hash_file_sha256(&summary_path)?,
    })
}

fn validate_values(
    summary: &BTreeMap<String, BenchmarkScientificValue>,
    values: &DuplicationValues,
    source: &str,
) -> Result<()> {
    let fraction = values.duplicate_reads as f64 / values.examined_reads as f64;
    if values.examined_reads == 0
        || values.examined_reads > MAX_EXACT_INTEGER
        || values.duplicate_reads > values.examined_reads
        || !values.duplication_fraction.is_finite()
        || values.duplication_fraction < 0.0
        || values.duplication_fraction > 1.0
        || (values.duplication_fraction - fraction).abs() > FLOAT_TOLERANCE
        || numeric_value(summary, "examined_reads")? != values.examined_reads as f64
        || numeric_value(summary, "duplicate_reads")? != values.duplicate_reads as f64
        || (numeric_value(summary, "duplication_fraction")? - values.duplication_fraction).abs()
            > FLOAT_TOLERANCE
    {
        bail!("BAM duplication-metrics {source} disagrees with normalized TSV");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        read_numeric_summary, validate_values, DuplicationBundle, DuplicationMetrics,
        DuplicationValues, METRICS,
    };

    #[test]
    fn duplication_values_allow_zero_duplicates_but_require_exact_fraction() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("bam-duplication-report")?;
        let path = temporary.path().join("summary.tsv");
        std::fs::write(
            &path,
            b"metric\tvalue\nexamined_reads\t10\nduplicate_reads\t0\nduplication_fraction\t0.000000000000\n",
        )?;
        let summary = read_numeric_summary(&path, &METRICS)?;
        let mut values =
            DuplicationValues { examined_reads: 10, duplicate_reads: 0, duplication_fraction: 0.0 };
        validate_values(&summary, &values, "test")?;
        values.duplicate_reads = 11;
        assert!(validate_values(&summary, &values, "test").is_err());
        Ok(())
    }

    #[test]
    fn typed_duplication_documents_accept_exact_contracts() -> anyhow::Result<()> {
        let mut document = serde_json::json!({
            "schema_version": "bijux.benchmark.bam.duplication_metrics.bundle.v1",
            "stage_id": "bam.duplication_metrics",
            "sample_id": "ERR2112767",
            "examined_reads": 10,
            "duplicate_reads": 2,
            "duplication_fraction": 0.2,
        });
        document
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("duplication-metrics fixture is not an object"))?
            .insert(["tool", "id"].join("_"), serde_json::json!("samtools"));
        let bundle: DuplicationBundle = serde_json::from_value(document)?;
        let metrics: DuplicationMetrics = serde_json::from_value(serde_json::json!({
            "examined_reads": 10,
            "duplicate_reads": 2,
            "duplication_fraction": 0.2,
            "bam_input_bytes": 23,
            "bam_index_input_bytes": 3,
            "input_bytes": 26,
            "output_bytes": 100,
        }))?;
        assert_eq!(bundle.values.duplicate_reads, metrics.values.duplicate_reads);
        Ok(())
    }
}
