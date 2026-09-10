//! Owner: bijux-dna-bench-population
//! Admit upstream cohort inputs and prepare explicitly owned stage paths.
use crate::roh::admission::cohort::validate_expected_cohort_samples;
use crate::roh::execution::artifacts::resolve_manifest_relative_path;
use crate::roh::model::DEFAULT_INPUT_SAMPLE_METADATA_NAME;
use crate::roh::model::DEFAULT_INPUT_VCF_NAME;
use crate::roh::model::DEFAULT_VCF_ROH_SMOKE_ROOT;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_corpus::vcf::load_sample_metadata;
use bijux_dna_bench_corpus::vcf::load_vcf_corpus_fixture_manifest_path;
use bijux_dna_bench_corpus::vcf::DEFAULT_VCF_MINI_MANIFEST_PATH;
use std::path::Path;

pub(crate) struct PreparedRohInputs {
    pub(crate) fixture_manifest_path: std::path::PathBuf,
    pub(crate) expected_samples: Vec<String>,
    pub(crate) output_root: std::path::PathBuf,
    pub(crate) stage_root: std::path::PathBuf,
    pub(crate) input_vcf_path: std::path::PathBuf,
    pub(crate) sample_metadata_path: std::path::PathBuf,
}

pub(crate) fn prepare_inputs(
    repo_root: &Path,
    contract: &crate::roh::model::GovernedVcfRohSmokeContract,
) -> Result<PreparedRohInputs> {
    let fixture_manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
    let fixture_manifest = load_vcf_corpus_fixture_manifest_path(&fixture_manifest_path)?;
    let fixture_dir = fixture_manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", fixture_manifest_path.display())
    })?;
    let input_vcf_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.multisample_vcf_path);
    let sample_metadata_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.sample_metadata_path);
    let expected_samples = fixture_manifest.expected_multisample_sample_ids.clone();
    if expected_samples.is_empty() {
        bail!("governed VCF ROH smoke fixture must declare multisample cohort sample ids");
    }
    let sample_metadata = load_sample_metadata(&sample_metadata_source)?;
    validate_expected_cohort_samples(&sample_metadata, &expected_samples)?;

    let output_root = repo_root.join(DEFAULT_VCF_ROH_SMOKE_ROOT).join(&contract.tool);
    bijux_dna_infra::ensure_dir(&output_root)
        .with_context(|| format!("create {}", output_root.display()))?;
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    bijux_dna_infra::ensure_dir(&input_root)
        .with_context(|| format!("create {}", input_root.display()))?;
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;

    let input_vcf_path = input_root.join(DEFAULT_INPUT_VCF_NAME);
    bijux_dna_infra::copy_file(&input_vcf_source, &input_vcf_path).with_context(|| {
        format!("copy {} to {}", input_vcf_source.display(), input_vcf_path.display())
    })?;
    let sample_metadata_path = input_root.join(DEFAULT_INPUT_SAMPLE_METADATA_NAME);
    bijux_dna_infra::copy_file(&sample_metadata_source, &sample_metadata_path).with_context(
        || {
            format!(
                "copy {} to {}",
                sample_metadata_source.display(),
                sample_metadata_path.display()
            )
        },
    )?;

    Ok(PreparedRohInputs {
        fixture_manifest_path,
        expected_samples,
        output_root,
        stage_root,
        input_vcf_path,
        sample_metadata_path,
    })
}
