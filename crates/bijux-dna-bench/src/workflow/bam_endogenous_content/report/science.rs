//! Owner: bijux-dna-bench
//! Typed scientific evidence validation for BAM endogenous-content reports.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Result};
use bijux_dna_bench_model::{BenchmarkExecutionSpec, BenchmarkScientificValue};
use serde::Deserialize;

use super::super::super::bam_observation::{
    numeric_value, read_numeric_summary, validate_byte_counts, ScientificObservation,
};
use crate::workflow::stage_report::read_json;

const BUNDLE_SCHEMA: &str = "bijux.benchmark.bam.endogenous_content.bundle.v1";
const STAGE_ID: &str = "bam.endogenous_content";
const PROBE_STAGE_ID: &str = "benchmark.bam.endogenous_content.consume";
const FLOAT_TOLERANCE: f64 = 1e-12;
const MAX_EXACT_INTEGER: u64 = 1 << 53;
const METRICS: [&str; 4] = [
    "total_primary_reads",
    "mapped_primary_reads",
    "mapped_fraction",
    "mapped_nonduplicate_primary_reads",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EndogenousValues {
    total_primary_reads: u64,
    mapped_primary_reads: u64,
    mapped_fraction: f64,
    mapped_nonduplicate_primary_reads: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EndogenousBundle {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_id: String,
    #[serde(flatten)]
    values: EndogenousValues,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EndogenousMetrics {
    #[serde(flatten)]
    values: EndogenousValues,
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
    let summary_path = root.join(format!("{prefix}.endogenous-content.tsv"));
    let summary = read_numeric_summary(&summary_path, &METRICS)?;
    let bundle: EndogenousBundle =
        read_json(&root.join(format!("{prefix}.endogenous-content.json")))?;
    let metrics: EndogenousMetrics = read_json(&root.join(format!("{prefix}.metrics.json")))?;
    let probe: DownstreamProbe = read_json(&root.join(format!("{prefix}.report-qc.json")))?;

    if bundle.schema_version != BUNDLE_SCHEMA
        || bundle.stage_id != STAGE_ID
        || bundle.tool_id != spec.identity.tool_id
        || bundle.sample_id != super::super::SAMPLE_ID
    {
        bail!("BAM endogenous-content bundle identity is invalid");
    }
    validate_values(&summary, &bundle.values, "bundle")?;
    validate_values(&summary, &metrics.values, "metrics")?;
    if probe.schema_version != "bijux.benchmark.downstream_probe.v1"
        || probe.stage_id != PROBE_STAGE_ID
        || probe.input_contract != BUNDLE_SCHEMA
        || !probe.complete
    {
        bail!("BAM endogenous-content downstream probe is not complete");
    }
    let raw = root.join(format!("{prefix}.counts.tsv"));
    let summary_json = root.join(format!("{prefix}.endogenous-content.json"));
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
    values: &EndogenousValues,
    source: &str,
) -> Result<()> {
    let fraction = values.mapped_primary_reads as f64 / values.total_primary_reads as f64;
    if values.total_primary_reads == 0
        || values.total_primary_reads > MAX_EXACT_INTEGER
        || values.mapped_primary_reads == 0
        || values.mapped_primary_reads > values.total_primary_reads
        || values.mapped_nonduplicate_primary_reads == 0
        || values.mapped_nonduplicate_primary_reads > values.mapped_primary_reads
        || !values.mapped_fraction.is_finite()
        || values.mapped_fraction <= 0.0
        || values.mapped_fraction > 1.0
        || (values.mapped_fraction - fraction).abs() > FLOAT_TOLERANCE
        || numeric_value(summary, "total_primary_reads")? != values.total_primary_reads as f64
        || numeric_value(summary, "mapped_primary_reads")? != values.mapped_primary_reads as f64
        || (numeric_value(summary, "mapped_fraction")? - values.mapped_fraction).abs()
            > FLOAT_TOLERANCE
        || numeric_value(summary, "mapped_nonduplicate_primary_reads")?
            != values.mapped_nonduplicate_primary_reads as f64
    {
        bail!("BAM endogenous-content {source} disagrees with normalized TSV");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        read_numeric_summary, validate_values, EndogenousBundle, EndogenousMetrics,
        EndogenousValues, METRICS,
    };

    #[test]
    fn endogenous_values_require_consistent_counts() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("bam-endogenous-report")?;
        let path = temporary.path().join("summary.tsv");
        std::fs::write(
            &path,
            b"metric\tvalue\ntotal_primary_reads\t10\nmapped_primary_reads\t8\nmapped_fraction\t0.800000000000\nmapped_nonduplicate_primary_reads\t7\n",
        )?;
        let summary = read_numeric_summary(&path, &METRICS)?;
        let mut values = EndogenousValues {
            total_primary_reads: 10,
            mapped_primary_reads: 8,
            mapped_fraction: 0.8,
            mapped_nonduplicate_primary_reads: 7,
        };
        validate_values(&summary, &values, "test")?;
        values.mapped_nonduplicate_primary_reads = 9;
        assert!(validate_values(&summary, &values, "test").is_err());
        Ok(())
    }

    #[test]
    fn typed_endogenous_documents_accept_exact_contracts() -> anyhow::Result<()> {
        let mut document = serde_json::json!({
            "schema_version": "bijux.benchmark.bam.endogenous_content.bundle.v1",
            "stage_id": "bam.endogenous_content",
            "sample_id": "ERR2112767",
            "total_primary_reads": 10,
            "mapped_primary_reads": 8,
            "mapped_fraction": 0.8,
            "mapped_nonduplicate_primary_reads": 7,
        });
        document
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("endogenous-content fixture is not an object"))?
            .insert(["tool", "id"].join("_"), serde_json::json!("samtools"));
        let bundle: EndogenousBundle = serde_json::from_value(document)?;
        let metrics: EndogenousMetrics = serde_json::from_value(serde_json::json!({
            "total_primary_reads": 10,
            "mapped_primary_reads": 8,
            "mapped_fraction": 0.8,
            "mapped_nonduplicate_primary_reads": 7,
            "bam_input_bytes": 23,
            "bam_index_input_bytes": 3,
            "input_bytes": 26,
            "output_bytes": 100,
        }))?;
        assert_eq!(bundle.values.total_primary_reads, metrics.values.total_primary_reads);
        Ok(())
    }
}
