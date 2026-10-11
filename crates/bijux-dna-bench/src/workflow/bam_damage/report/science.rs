//! Owner: bijux-dna-bench
//! Typed evidence validation for BAM damage qualification reports.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::normalized_result::{
    bam::{
        BenchmarkBamDamageMetrics as DamageMetrics, BenchmarkBamDamageResult as DamageBundle,
        BAM_DAMAGE_RESULT_SCHEMA_V1 as BUNDLE_SCHEMA,
    },
    BenchmarkDownstreamProbeRecord as DownstreamProbe,
};
use bijux_dna_bench_model::{BenchmarkExecutionSpec, BenchmarkScientificValue};

use super::super::super::bam_observation::{
    numeric_value, read_numeric_summary, ScientificObservation,
};
use crate::workflow::stage_report::read_json;

const PROBE_STAGE_ID: &str = "benchmark.bam.damage.consume";
const MAX_EXACT_INTEGER: u64 = 1 << 53;
const METRICS: [&str; 3] = ["normalization_complete", "native_file_count", "native_output_bytes"];

pub(super) fn derive_scientific_observation(
    spec: &BenchmarkExecutionSpec,
    repetition: u32,
) -> Result<ScientificObservation> {
    let root = Path::new(&spec.outputs.result_root);
    let prefix = format!("repetitions/repetition-{repetition}");
    let native_path = root.join(format!("{prefix}.native"));
    let raw_path = root.join(format!("{prefix}.damage-native-summary.tsv"));
    let summary_path = root.join(format!("{prefix}.damage-summary.tsv"));
    let bundle_path = root.join(format!("{prefix}.damage-summary.json"));
    let probe_path = root.join(format!("{prefix}.report-qc.json"));
    let summary = read_numeric_summary(&summary_path, &METRICS)?;
    let bundle: DamageBundle = read_json(&bundle_path)?;
    let metrics: DamageMetrics = read_json(&root.join(format!("{prefix}.metrics.json")))?;
    let probe: DownstreamProbe = read_json(&probe_path)?;

    if bundle.schema_version != BUNDLE_SCHEMA
        || bundle.stage_id != super::super::STAGE_ID
        || bundle.tool_id != spec.identity.tool_id
        || bundle.sample_id != super::super::SAMPLE_ID
        || bundle.authenticity_certified
    {
        bail!("BAM damage bundle identity or qualification scope is invalid");
    }
    validate_values(&summary, &bundle, &metrics)?;
    if probe.schema_version != "bijux.benchmark.downstream_probe.v1"
        || probe.stage_id != PROBE_STAGE_ID
        || probe.input_contract != BUNDLE_SCHEMA
        || !probe.complete
        || probe.authenticity_certified != Some(false)
    {
        bail!("BAM damage downstream probe exceeds or violates qualification scope");
    }
    let (native_file_count, native_output_bytes) = native_evidence_size(&native_path)?;
    if metrics.native_file_count != native_file_count
        || metrics.native_output_bytes != native_output_bytes
    {
        bail!("BAM damage native evidence accounting differs from preserved outputs");
    }
    validate_byte_counts(
        spec,
        &metrics,
        native_output_bytes,
        &[&raw_path, &summary_path, &bundle_path, &probe_path],
    )?;
    Ok(ScientificObservation {
        values: summary,
        bam_input_bytes: metrics.bam_input_bytes,
        bam_index_input_bytes: metrics.bam_index_input_bytes,
        reference_input_bytes: metrics.reference_input_bytes,
        input_bytes: metrics.input_bytes,
        output_bytes: metrics.output_bytes,
        normalized_bundle_sha256: bijux_dna_infra::hash_file_sha256(&summary_path)?,
    })
}

fn validate_values(
    summary: &BTreeMap<String, BenchmarkScientificValue>,
    bundle: &DamageBundle,
    metrics: &DamageMetrics,
) -> Result<()> {
    if bundle.native_file_count > MAX_EXACT_INTEGER
        || bundle.native_output_bytes > MAX_EXACT_INTEGER
        || metrics.native_file_count > MAX_EXACT_INTEGER
        || metrics.native_output_bytes > MAX_EXACT_INTEGER
        || bundle.normalization_complete != metrics.normalization_complete
        || bundle.native_file_count != metrics.native_file_count
        || bundle.native_output_bytes != metrics.native_output_bytes
        || numeric_value(summary, "normalization_complete")?
            != f64::from(bundle.normalization_complete)
        || numeric_value(summary, "native_file_count")? != bundle.native_file_count as f64
        || numeric_value(summary, "native_output_bytes")? != bundle.native_output_bytes as f64
    {
        bail!("BAM damage normalized documents disagree with their TSV authority");
    }
    Ok(())
}

fn validate_byte_counts(
    spec: &BenchmarkExecutionSpec,
    metrics: &DamageMetrics,
    native_output_bytes: u64,
    normalized_outputs: &[&Path],
) -> Result<()> {
    let bam = input_size(spec, "bam")?;
    let bam_index = input_size(spec, "bam_index")?;
    let reference = input_size(spec, "reference")?
        .checked_add(input_size(spec, "reference_index")?)
        .context("BAM damage reference size overflow")?;
    let input_bytes = bam
        .checked_add(bam_index)
        .and_then(|value| value.checked_add(reference))
        .context("BAM damage input size overflow")?;
    let output_bytes = normalized_outputs.iter().try_fold(native_output_bytes, |total, path| {
        total.checked_add(regular_file_size(path)?).context("BAM damage output size overflow")
    })?;
    if metrics.bam_input_bytes != bam
        || metrics.bam_index_input_bytes != bam_index
        || metrics.reference_input_bytes != reference
        || metrics.input_bytes != input_bytes
        || metrics.output_bytes != output_bytes
    {
        bail!("BAM damage metrics contain invalid input or output byte counts");
    }
    Ok(())
}

fn native_evidence_size(path: &Path) -> Result<(u64, u64)> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect BAM damage native evidence {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("BAM damage native evidence is not a non-symlink directory");
    }
    let mut count = 0_u64;
    let mut bytes = 0_u64;
    for entry in walkdir::WalkDir::new(path).follow_links(false).min_depth(1) {
        let entry = entry.context("walk BAM damage native evidence")?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            bail!("BAM damage native evidence contains a symbolic link");
        }
        if !metadata.is_file() && !metadata.is_dir() {
            bail!("BAM damage native evidence contains a non-regular filesystem entry");
        }
        if metadata.is_file() && metadata.len() > 0 {
            count = count.checked_add(1).context("BAM damage native file count overflow")?;
            bytes = bytes.checked_add(metadata.len()).context("BAM damage native size overflow")?;
        }
    }
    if count == 0 || bytes == 0 {
        bail!("BAM damage native evidence is empty");
    }
    Ok((count, bytes))
}

fn input_size(spec: &BenchmarkExecutionSpec, id: &str) -> Result<u64> {
    let input = spec
        .inputs
        .iter()
        .find(|input| input.id == id)
        .with_context(|| format!("BAM damage execution omits {id}"))?;
    regular_file_size(Path::new(&input.path))
}

fn regular_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect BAM damage evidence {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() == 0 {
        bail!("BAM damage evidence is not a nonempty regular non-symlink file");
    }
    Ok(metadata.len())
}

#[cfg(test)]
mod tests {
    use super::{native_evidence_size, validate_values, DamageBundle, DamageMetrics, METRICS};

    #[test]
    fn typed_damage_documents_preserve_qualification_only_semantics() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("bam-damage-report")?;
        let summary_path = temporary.path().join("damage-summary.tsv");
        std::fs::write(
            &summary_path,
            b"metric\tvalue\nnormalization_complete\t1\nnative_file_count\t2\nnative_output_bytes\t19\n",
        )?;
        let summary = super::read_numeric_summary(&summary_path, &METRICS)?;
        let bundle: DamageBundle = serde_json::from_value(serde_json::json!({
            "schema_version": "bijux.benchmark.bam.damage.bundle.v1",
            "stage_id": "bam.damage",
            "tool_id": "ngsbriggs",
            "sample_id": "ERR2112767",
            "normalization_complete": 1,
            "native_file_count": 2,
            "native_output_bytes": 19,
            "authenticity_certified": false
        }))?;
        let metrics: DamageMetrics = serde_json::from_value(serde_json::json!({
            "schema_version": "bijux.benchmark.bam.damage.metrics.v1",
            "normalization_complete": 1,
            "native_file_count": 2,
            "native_output_bytes": 19,
            "bam_input_bytes": 100,
            "bam_index_input_bytes": 10,
            "reference_input_bytes": 220,
            "input_bytes": 330,
            "output_bytes": 300
        }))?;
        validate_values(&summary, &bundle, &metrics)?;
        assert!(!bundle.authenticity_certified);
        Ok(())
    }

    #[test]
    fn native_evidence_accounting_is_recursive_and_nonempty() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("bam-damage-native-evidence")?;
        let nested = temporary.path().join("nested");
        std::fs::create_dir(&nested)?;
        std::fs::write(temporary.path().join("root.txt"), b"123")?;
        std::fs::write(nested.join("child.txt"), b"12345")?;
        std::fs::write(nested.join("empty.txt"), b"")?;

        assert_eq!(native_evidence_size(temporary.path())?, (2, 8));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn native_evidence_rejects_symbolic_links() -> anyhow::Result<()> {
        use std::os::unix::fs::symlink;

        let temporary = bijux_dna_infra::temp_dir("bam-damage-native-symlink")?;
        let target = temporary.path().join("target.txt");
        std::fs::write(&target, b"evidence")?;
        symlink(&target, temporary.path().join("alias.txt"))?;

        assert!(native_evidence_size(temporary.path()).is_err());
        Ok(())
    }
}
