//! Admit expected sex-inference wire and insufficiency contracts.
use super::class_coverage::{collect_call_classes, validate_required_call_classes};
use super::model::{BamSexSampleTruth, BamSexTruthBundle, BamSexTruthManifest};
use super::BAM_SEX_TRUTH_BUNDLE_SCHEMA_VERSION;
use anyhow::{anyhow, Result};
use bijux_dna_core::id_catalog::BAM_SEX;
use bijux_dna_domain_bam::metrics::SexConfidenceClass;
use bijux_dna_domain_bam::BAM_SEX_SUMMARY_SCHEMA_VERSION;
use std::path::Path;

pub(super) fn validate_bundle_contract(
    manifest: &BamSexTruthManifest,
    bundle: &BamSexTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_SEX_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM sex truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_SEX_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM sex truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let actual_sample_ids =
        bundle.sample_truths.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>();
    let expected_sample_ids =
        manifest.cases.iter().map(|case| case.sample_id.as_str()).collect::<Vec<_>>();
    if actual_sample_ids != expected_sample_ids {
        return Err(anyhow!(
            "BAM sex truth bundle `{}` must contain samples {:?}",
            expected_path.display(),
            expected_sample_ids
        ));
    }
    for sample in &bundle.sample_truths {
        validate_sample_truth_contract(sample, expected_path)?;
    }
    validate_required_call_classes(&collect_call_classes(&bundle.sample_truths), expected_path)?;
    Ok(())
}

fn validate_sample_truth_contract(sample: &BamSexSampleTruth, expected_path: &Path) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth bundle `{}` contains a sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() || sample.reference_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must declare non-empty input paths",
            sample.sample_id
        ));
    }
    if sample.summary.summary_schema_version != BAM_SEX_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM sex truth sample `{}` uses schema `{}` instead of `{}`",
            sample.sample_id,
            sample.summary.summary_schema_version,
            BAM_SEX_SUMMARY_SCHEMA_VERSION
        ));
    }
    if sample.summary.stage_id != BAM_SEX {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must keep stage_id `bam.sex`",
            sample.sample_id
        ));
    }
    if sample.summary.method.trim().is_empty() || sample.summary.status.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must declare non-empty method and status",
            sample.sample_id
        ));
    }
    if sample.summary.x_contig.trim().is_empty() || sample.summary.y_contig.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must declare non-empty X/Y contig names",
            sample.sample_id
        ));
    }
    if sample.summary.x_coverage < 0.0
        || sample.summary.y_coverage < 0.0
        || sample.summary.autosomal_coverage < 0.0
    {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must keep non-negative coverage values",
            sample.sample_id
        ));
    }
    if sample.summary.call == SexConfidenceClass::Insufficient {
        if sample.summary.insufficiency_reason.as_deref().unwrap_or_default().trim().is_empty() {
            return Err(anyhow!(
                "BAM sex truth sample `{}` must declare insufficiency_reason when call is insufficient",
                sample.sample_id
            ));
        }
        if sample.summary.confidence != 0.0 {
            return Err(anyhow!(
                "BAM sex truth sample `{}` must keep confidence zero when call is insufficient",
                sample.sample_id
            ));
        }
    } else if sample.summary.status == "ok" && sample.summary.insufficiency_reason.is_some() {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must not declare insufficiency_reason when status is ok",
            sample.sample_id
        ));
    }
    Ok(())
}
