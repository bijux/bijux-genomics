//! Owner: bijux-dna-bench
//! Typed scientific evidence validation for BAM coverage reports.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use bijux_dna_bench_model::{BenchmarkExecutionSpec, BenchmarkScientificValue};
use serde::Deserialize;

use super::super::super::bam_observation::{
    numeric_value, read_numeric_summary, validate_byte_counts, ScientificObservation,
};
use crate::workflow::stage_report::read_json;

const BUNDLE_SCHEMA: &str = "bijux.benchmark.bam.coverage.bundle.v1";
const STAGE_ID: &str = "bam.coverage";
const PROBE_STAGE_ID: &str = "benchmark.bam.coverage.compare";
const FLOAT_TOLERANCE: f64 = 1e-12;
const MAX_EXACT_INTEGER: u64 = 1 << 53;
const METRICS: [&str; 4] = ["reference_bases", "covered_bases", "coverage_fraction", "mean_depth"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageValues {
    reference_bases: u64,
    covered_bases: u64,
    coverage_fraction: f64,
    mean_depth: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageBundle {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_id: String,
    #[serde(flatten)]
    values: CoverageValues,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageMetrics {
    #[serde(flatten)]
    values: CoverageValues,
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
    let summary_path = root.join(format!("{prefix}.coverage-summary.tsv"));
    let summary = read_numeric_summary(&summary_path, &METRICS)?;
    let bundle: CoverageBundle = read_json(&root.join(format!("{prefix}.coverage-summary.json")))?;
    let metrics: CoverageMetrics = read_json(&root.join(format!("{prefix}.metrics.json")))?;
    let probe: DownstreamProbe = read_json(&root.join(format!("{prefix}.report-qc.json")))?;

    if bundle.schema_version != BUNDLE_SCHEMA
        || bundle.stage_id != STAGE_ID
        || bundle.tool_id != spec.identity.tool_id
        || bundle.sample_id != super::super::SAMPLE_ID
    {
        bail!("BAM coverage bundle identity is invalid");
    }
    validate_values(&summary, &bundle.values, "bundle")?;
    validate_values(&summary, &metrics.values, "metrics")?;
    if probe.schema_version != "bijux.benchmark.downstream_probe.v1"
        || probe.stage_id != PROBE_STAGE_ID
        || probe.input_contract != BUNDLE_SCHEMA
        || !probe.complete
    {
        bail!("BAM coverage downstream probe is not complete");
    }
    let outputs = output_paths(root, &prefix, &spec.identity.tool_id)?;
    validate_byte_counts(
        spec,
        metrics.bam_input_bytes,
        metrics.bam_index_input_bytes,
        metrics.input_bytes,
        metrics.output_bytes,
        &outputs.iter().map(PathBuf::as_path).collect::<Vec<_>>(),
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

fn output_paths(root: &Path, prefix: &str, tool: &str) -> Result<Vec<PathBuf>> {
    let mut paths = match tool {
        "bedtools" => vec![root.join(format!("{prefix}.bedgraph"))],
        "mosdepth" => vec![
            root.join(format!("{prefix}.mosdepth.summary.txt")),
            root.join(format!("{prefix}.mosdepth.global.dist.txt")),
        ],
        "samtools" => vec![
            root.join(format!("{prefix}.depth.tsv")),
            root.join(format!("{prefix}.reference-span.tsv")),
        ],
        _ => bail!("BAM coverage report encountered an undeclared tool"),
    };
    paths.extend([
        root.join(format!("{prefix}.coverage-summary.tsv")),
        root.join(format!("{prefix}.coverage-summary.json")),
        root.join(format!("{prefix}.report-qc.json")),
    ]);
    Ok(paths)
}

fn validate_values(
    summary: &BTreeMap<String, BenchmarkScientificValue>,
    values: &CoverageValues,
    source: &str,
) -> Result<()> {
    let expected_fraction = values.covered_bases as f64 / values.reference_bases as f64;
    if values.reference_bases == 0
        || values.reference_bases > MAX_EXACT_INTEGER
        || values.covered_bases == 0
        || values.covered_bases > values.reference_bases
        || !values.coverage_fraction.is_finite()
        || values.coverage_fraction <= 0.0
        || values.coverage_fraction > 1.0
        || (values.coverage_fraction - expected_fraction).abs() > FLOAT_TOLERANCE
        || !values.mean_depth.is_finite()
        || values.mean_depth <= 0.0
        || numeric_value(summary, "reference_bases")? != values.reference_bases as f64
        || numeric_value(summary, "covered_bases")? != values.covered_bases as f64
        || (numeric_value(summary, "coverage_fraction")? - values.coverage_fraction).abs()
            > FLOAT_TOLERANCE
        || (numeric_value(summary, "mean_depth")? - values.mean_depth).abs() > FLOAT_TOLERANCE
    {
        bail!("BAM coverage {source} disagrees with normalized TSV");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        read_numeric_summary, validate_values, CoverageBundle, CoverageMetrics, CoverageValues,
        METRICS,
    };

    #[test]
    fn coverage_values_require_exact_fraction() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("bam-coverage-report")?;
        let path = temporary.path().join("summary.tsv");
        std::fs::write(
            &path,
            b"metric\tvalue\nreference_bases\t10\ncovered_bases\t6\ncoverage_fraction\t0.600000000000\nmean_depth\t1.200000000000\n",
        )?;
        let summary = read_numeric_summary(&path, &METRICS)?;
        let mut values = CoverageValues {
            reference_bases: 10,
            covered_bases: 6,
            coverage_fraction: 0.6,
            mean_depth: 1.2,
        };
        validate_values(&summary, &values, "test")?;
        values.coverage_fraction = 0.7;
        assert!(validate_values(&summary, &values, "test").is_err());
        Ok(())
    }

    #[test]
    fn typed_coverage_documents_accept_exact_contracts() -> anyhow::Result<()> {
        let mut document = serde_json::json!({
            "schema_version": "bijux.benchmark.bam.coverage.bundle.v1",
            "stage_id": "bam.coverage",
            "sample_id": "ERR2112767",
            "reference_bases": 10,
            "covered_bases": 6,
            "coverage_fraction": 0.6,
            "mean_depth": 1.2,
        });
        document
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("coverage fixture is not an object"))?
            .insert(["tool", "id"].join("_"), serde_json::json!("samtools"));
        let bundle: CoverageBundle = serde_json::from_value(document)?;
        let metrics: CoverageMetrics = serde_json::from_value(serde_json::json!({
            "reference_bases": 10,
            "covered_bases": 6,
            "coverage_fraction": 0.6,
            "mean_depth": 1.2,
            "bam_input_bytes": 23,
            "bam_index_input_bytes": 3,
            "input_bytes": 26,
            "output_bytes": 100,
        }))?;
        assert_eq!(bundle.values.reference_bases, metrics.values.reference_bases);
        Ok(())
    }
}
