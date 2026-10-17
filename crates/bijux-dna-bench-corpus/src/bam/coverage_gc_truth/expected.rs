//! Admit expected coverage aggregates and GC-bin/dropout invariants.
use super::model::{
    BamCoverageSampleTruth, BamGcBiasSampleTruth, BamGcCoverageTruthBundle,
    BamGcCoverageTruthManifest,
};
use super::BAM_GC_COVERAGE_TRUTH_BUNDLE_SCHEMA_VERSION;
use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use bijux_dna_domain_bam::{
    BAM_COVERAGE_TRUTH_SUMMARY_SCHEMA_VERSION, BAM_GC_BIAS_TRUTH_SUMMARY_SCHEMA_VERSION,
};
use std::path::Path;

pub(super) fn validate_bundle_contract(
    manifest: &BamGcCoverageTruthManifest,
    bundle: &BamGcCoverageTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_GC_COVERAGE_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_GC_COVERAGE_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }

    let actual_coverage_ids =
        bundle.coverage_truths.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>();
    let expected_coverage_ids =
        manifest.coverage_cases.iter().map(|case| case.sample_id.as_str()).collect::<Vec<_>>();
    if actual_coverage_ids != expected_coverage_ids {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle `{}` must contain coverage samples {:?}",
            expected_path.display(),
            expected_coverage_ids
        ));
    }

    let actual_gc_bias_ids =
        bundle.gc_bias_truths.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>();
    let expected_gc_bias_ids =
        manifest.gc_bias_cases.iter().map(|case| case.sample_id.as_str()).collect::<Vec<_>>();
    if actual_gc_bias_ids != expected_gc_bias_ids {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle `{}` must contain GC-bias samples {:?}",
            expected_path.display(),
            expected_gc_bias_ids
        ));
    }

    for sample in &bundle.coverage_truths {
        validate_coverage_sample_truth_contract(sample, expected_path)?;
    }
    for sample in &bundle.gc_bias_truths {
        validate_gc_bias_sample_truth_contract(sample, expected_path)?;
    }
    Ok(())
}

fn validate_coverage_sample_truth_contract(
    sample: &BamCoverageSampleTruth,
    expected_path: &Path,
) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle `{}` contains a coverage sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() || sample.regions_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare non-empty input paths",
            sample.sample_id
        ));
    }
    if sample.summary.schema_version != BAM_COVERAGE_TRUTH_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` uses schema `{}` instead of `{}`",
            sample.sample_id,
            sample.summary.schema_version,
            BAM_COVERAGE_TRUTH_SUMMARY_SCHEMA_VERSION
        ));
    }
    if sample.summary.depth_thresholds.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare depth thresholds",
            sample.sample_id
        ));
    }
    if sample.summary.region_summaries.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare region summaries",
            sample.sample_id
        ));
    }
    if sample.summary.coverage_regime.trim().is_empty()
        || sample.summary.coverage_family.trim().is_empty()
    {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare non-empty coverage labels",
            sample.sample_id
        ));
    }

    let total_bases = sample.summary.region_summaries.iter().map(|row| row.length).sum::<u64>();
    let covered_bases =
        sample.summary.region_summaries.iter().map(|row| row.covered_bases).sum::<u64>();
    if sample.summary.total_bases != total_bases {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` total_bases {} does not match region total {}",
            sample.sample_id,
            sample.summary.total_bases,
            total_bases
        ));
    }
    if sample.summary.covered_bases != covered_bases {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` covered_bases {} does not match region total {}",
            sample.sample_id,
            sample.summary.covered_bases,
            covered_bases
        ));
    }

    let expected_breadth = if total_bases == 0 {
        0.0
    } else {
        checked_f64_from_u64(covered_bases, "covered bases")?
            / checked_f64_from_u64(total_bases, "total bases")?
    };
    if !approx_eq(sample.summary.breadth_1x, expected_breadth) {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` breadth_1x {} does not match region breadth {}",
            sample.sample_id,
            sample.summary.breadth_1x,
            expected_breadth
        ));
    }

    let weighted_depth_sum = sample.summary.region_summaries.iter().try_fold(0.0, |sum, row| {
        Ok::<f64, anyhow::Error>(
            sum + row.mean_depth * checked_f64_from_u64(row.length, "region length")?,
        )
    })?;
    let expected_mean_depth = if total_bases == 0 {
        0.0
    } else {
        weighted_depth_sum / checked_f64_from_u64(total_bases, "total bases")?
    };
    if !approx_eq(sample.summary.mean_depth, expected_mean_depth) {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` mean_depth {} does not match weighted region mean {}",
            sample.sample_id,
            sample.summary.mean_depth,
            expected_mean_depth
        ));
    }

    Ok(())
}

fn validate_gc_bias_sample_truth_contract(
    sample: &BamGcBiasSampleTruth,
    expected_path: &Path,
) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle `{}` contains a GC-bias sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() || sample.reference_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` must declare non-empty input paths",
            sample.sample_id
        ));
    }
    if sample.summary.schema_version != BAM_GC_BIAS_TRUTH_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` uses schema `{}` instead of `{}`",
            sample.sample_id,
            sample.summary.schema_version,
            BAM_GC_BIAS_TRUTH_SUMMARY_SCHEMA_VERSION
        ));
    }
    if sample.summary.window_size == 0 {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` must declare window_size greater than zero",
            sample.sample_id
        ));
    }
    if sample.summary.gc_bins.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` must declare GC bins",
            sample.sample_id
        ));
    }

    let windows = sample.summary.gc_bins.iter().map(|row| row.windows).sum::<u64>();
    let read_starts = sample.summary.gc_bins.iter().map(|row| row.read_starts).sum::<u64>();
    if sample.summary.windows != windows {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` windows {} does not match GC-bin total {}",
            sample.sample_id,
            sample.summary.windows,
            windows
        ));
    }
    if sample.summary.read_starts != read_starts {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` read_starts {} does not match GC-bin total {}",
            sample.sample_id,
            sample.summary.read_starts,
            read_starts
        ));
    }

    let expected_gc_bias_score = sample.summary.at_dropout.max(sample.summary.gc_dropout) / 100.0;
    if !approx_eq(sample.summary.gc_bias_score, expected_gc_bias_score) {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` gc_bias_score {} does not match dropout-derived score {}",
            sample.sample_id,
            sample.summary.gc_bias_score,
            expected_gc_bias_score
        ));
    }

    Ok(())
}

fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}
