//! Owner: bijux-dna-bench
//! Contract-aware scientific derivation for BAM genotyping reports.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkExecutionSpec, BenchmarkMetricDistribution, BenchmarkScientificValue,
};
use serde::Deserialize;

use super::BUNDLE_SCHEMA;
use crate::workflow::bam_observation::{numeric_value, read_numeric_summary};
use crate::workflow::stage_report::{
    distribution, read_json, ResourceObservation, MAX_REPORT_INPUT_BYTES,
};

const FLOAT_TOLERANCE: f64 = 1e-12;
const MAX_EXACT_INTEGER: u64 = 1 << 53;
const SCIENTIFIC_METRICS: [&str; 5] = [
    "total_records",
    "covered_sites",
    "called_genotypes",
    "alternate_genotypes",
    "mean_depth_at_covered_sites",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GenotypingBundle {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_id: String,
    total_records: u64,
    covered_sites: u64,
    called_genotypes: u64,
    alternate_genotypes: u64,
    mean_depth_at_covered_sites: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GenotypingMetrics {
    total_records: u64,
    covered_sites: u64,
    called_genotypes: u64,
    alternate_genotypes: u64,
    mean_depth_at_covered_sites: f64,
    bam_input_bytes: u64,
    reference_input_bytes: u64,
    database_input_bytes: u64,
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

#[derive(Debug, Clone)]
pub(super) struct ScientificObservation {
    pub(super) values: BTreeMap<String, BenchmarkScientificValue>,
    pub(super) bam_input_bytes: u64,
    pub(super) reference_input_bytes: u64,
    pub(super) database_input_bytes: u64,
    pub(super) input_bytes: u64,
    pub(super) output_bytes: u64,
    pub(super) genotypes_sha256: String,
}

pub(super) fn derive_scientific_observation(
    spec: &BenchmarkExecutionSpec,
    repetition: u32,
) -> Result<ScientificObservation> {
    let result_root = Path::new(&spec.outputs.result_root);
    let prefix = format!("repetitions/repetition-{repetition}");
    let summary_path = result_root.join(format!("{prefix}.genotyping-summary.tsv"));
    let genotype_path = result_root.join(format!("{prefix}.genotypes.tsv"));
    let vcf_path = result_root.join(format!("{prefix}.vcf.gz"));
    let vcf_index_path = result_root.join(format!("{prefix}.vcf.gz.csi"));
    let bundle_path = result_root.join(format!("{prefix}.genotyping-summary.json"));
    let probe_path = result_root.join(format!("{prefix}.report-qc.json"));
    let digest_path = result_root.join(format!("{prefix}.genotypes.sha256"));
    let summary = read_numeric_summary(&summary_path, &SCIENTIFIC_METRICS)?;
    let bundle: GenotypingBundle = read_json(&bundle_path)?;
    let metrics: GenotypingMetrics =
        read_json(&result_root.join(format!("{prefix}.metrics.json")))?;
    let probe: DownstreamProbe = read_json(&probe_path)?;
    validate_bundle(spec, &bundle, &summary)?;
    validate_metrics(&metrics, &summary)?;
    if probe.schema_version != "bijux.benchmark.downstream_probe.v1"
        || probe.stage_id != "benchmark.bam.genotyping.consume"
        || probe.input_contract != BUNDLE_SCHEMA
        || !probe.complete
    {
        bail!("BAM genotyping downstream probe is not complete");
    }
    let bam_input_bytes = input_sizes(spec, &["bam", "bam_index"])?;
    let reference_input_bytes = input_sizes(spec, &["reference", "reference_index"])?;
    let database_input_bytes = input_sizes(
        spec,
        &["known_sites", "known_sites_index", "candidate_regions", "constrained_alleles"],
    )?;
    let input_bytes = bam_input_bytes
        .checked_add(reference_input_bytes)
        .and_then(|value| value.checked_add(database_input_bytes))
        .context("BAM genotyping input size overflow")?;
    let output_bytes = [
        vcf_path.as_path(),
        vcf_index_path.as_path(),
        genotype_path.as_path(),
        summary_path.as_path(),
        bundle_path.as_path(),
        probe_path.as_path(),
        digest_path.as_path(),
    ]
    .iter()
    .try_fold(0_u64, |total, path| {
        total.checked_add(regular_file_size(path)?).context("BAM genotyping output size overflow")
    })?;
    if metrics.bam_input_bytes != bam_input_bytes
        || metrics.reference_input_bytes != reference_input_bytes
        || metrics.database_input_bytes != database_input_bytes
        || metrics.input_bytes != input_bytes
        || metrics.output_bytes == 0
        || metrics.output_bytes != output_bytes
    {
        bail!("BAM genotyping metrics contain invalid byte counts");
    }
    let genotypes_sha256 = bijux_dna_infra::hash_file_sha256(&genotype_path)?;
    validate_digest_file(&digest_path, &genotypes_sha256)?;
    Ok(ScientificObservation {
        values: summary,
        bam_input_bytes,
        reference_input_bytes,
        database_input_bytes,
        input_bytes,
        output_bytes,
        genotypes_sha256,
    })
}

fn validate_bundle(
    spec: &BenchmarkExecutionSpec,
    bundle: &GenotypingBundle,
    summary: &BTreeMap<String, BenchmarkScientificValue>,
) -> Result<()> {
    if bundle.schema_version != BUNDLE_SCHEMA
        || bundle.stage_id != super::super::STAGE_ID
        || bundle.tool_id != spec.identity.tool_id
        || bundle.sample_id != super::super::SAMPLE_ID
    {
        bail!("BAM genotyping bundle identity is invalid");
    }
    validate_values(
        summary,
        bundle.total_records,
        bundle.covered_sites,
        bundle.called_genotypes,
        bundle.alternate_genotypes,
        bundle.mean_depth_at_covered_sites,
        "bundle",
    )
}

fn validate_metrics(
    metrics: &GenotypingMetrics,
    summary: &BTreeMap<String, BenchmarkScientificValue>,
) -> Result<()> {
    validate_values(
        summary,
        metrics.total_records,
        metrics.covered_sites,
        metrics.called_genotypes,
        metrics.alternate_genotypes,
        metrics.mean_depth_at_covered_sites,
        "metrics",
    )
}

#[allow(clippy::too_many_arguments)]
fn validate_values(
    summary: &BTreeMap<String, BenchmarkScientificValue>,
    total_records: u64,
    covered_sites: u64,
    called_genotypes: u64,
    alternate_genotypes: u64,
    mean_depth: f64,
    source: &str,
) -> Result<()> {
    if total_records == 0
        || covered_sites == 0
        || called_genotypes == 0
        || covered_sites > total_records
        || called_genotypes > covered_sites
        || alternate_genotypes > called_genotypes
        || [total_records, covered_sites, called_genotypes, alternate_genotypes]
            .into_iter()
            .any(|value| value > MAX_EXACT_INTEGER)
        || !mean_depth.is_finite()
        || mean_depth <= 0.0
        || numeric_value(summary, "total_records")? != total_records as f64
        || numeric_value(summary, "covered_sites")? != covered_sites as f64
        || numeric_value(summary, "called_genotypes")? != called_genotypes as f64
        || numeric_value(summary, "alternate_genotypes")? != alternate_genotypes as f64
        || (numeric_value(summary, "mean_depth_at_covered_sites")? - mean_depth).abs()
            > FLOAT_TOLERANCE
    {
        bail!("BAM genotyping {source} disagrees with normalized TSV");
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

fn input_sizes(spec: &BenchmarkExecutionSpec, ids: &[&str]) -> Result<u64> {
    ids.iter().try_fold(0_u64, |total, id| {
        let input = spec
            .inputs
            .iter()
            .find(|input| input.id == *id)
            .with_context(|| format!("BAM genotyping execution omits {id}"))?;
        total
            .checked_add(regular_file_size(Path::new(&input.path))?)
            .context("BAM genotyping input category size overflow")
    })
}

fn validate_digest_file(path: &Path, expected: &str) -> Result<()> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)
        .with_context(|| format!("read BAM genotyping digest {}", path.display()))?;
    let text = std::str::from_utf8(&bytes).context("BAM genotyping digest is not UTF-8")?;
    let mut fields = text.split_whitespace();
    let digest = fields.next().context("BAM genotyping digest is empty")?;
    let artifact = fields.next().context("BAM genotyping digest omits artifact path")?;
    if fields.next().is_some()
        || digest != expected
        || digest.len() != 64
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || !artifact.ends_with(".genotypes.tsv")
    {
        bail!("BAM genotyping digest evidence is invalid");
    }
    Ok(())
}

fn regular_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect BAM genotyping evidence {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("BAM genotyping evidence is not a regular non-symlink file");
    }
    Ok(metadata.len())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::BenchmarkScientificValue;

    use super::validate_values;

    #[test]
    fn genotyping_values_enforce_candidate_site_accounting() {
        let summary = BTreeMap::from([
            ("total_records".to_string(), BenchmarkScientificValue::Number(100.0)),
            ("covered_sites".to_string(), BenchmarkScientificValue::Number(20.0)),
            ("called_genotypes".to_string(), BenchmarkScientificValue::Number(18.0)),
            ("alternate_genotypes".to_string(), BenchmarkScientificValue::Number(3.0)),
            ("mean_depth_at_covered_sites".to_string(), BenchmarkScientificValue::Number(1.25)),
        ]);
        assert!(validate_values(&summary, 100, 20, 18, 3, 1.25, "test").is_ok());
        assert!(validate_values(&summary, 100, 20, 21, 3, 1.25, "test").is_err());
        assert!(validate_values(&summary, 100, 20, 18, 19, 1.25, "test").is_err());
    }
}
