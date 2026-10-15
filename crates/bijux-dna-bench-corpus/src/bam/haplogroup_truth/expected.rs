//! Admit expected case headers, marker support and status-specific refusal semantics.
use super::model::{BamHaplogroupCaseTruth, BamHaplogroupTruthBundle, BamHaplogroupTruthManifest};
use super::status_coverage::{collect_statuses, validate_required_statuses};
use super::BAM_HAPLOGROUP_TRUTH_BUNDLE_SCHEMA_VERSION;
use anyhow::{anyhow, Result};
use bijux_dna_domain_bam::BAM_HAPLOGROUP_TRUTH_SUMMARY_SCHEMA_VERSION;
use std::path::Path;

pub(super) fn validate_bundle_contract(
    manifest: &BamHaplogroupTruthManifest,
    bundle: &BamHaplogroupTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_HAPLOGROUP_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM haplogroup truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_HAPLOGROUP_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM haplogroup truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let actual_case_ids =
        bundle.case_truths.iter().map(|case| case.case_id.as_str()).collect::<Vec<_>>();
    let expected_case_ids =
        manifest.cases.iter().map(|case| case.case_id.as_str()).collect::<Vec<_>>();
    if actual_case_ids != expected_case_ids {
        return Err(anyhow!(
            "BAM haplogroup truth bundle `{}` must contain cases {:?}",
            expected_path.display(),
            expected_case_ids
        ));
    }
    for case_truth in &bundle.case_truths {
        validate_case_truth_contract(case_truth, expected_path)?;
    }
    validate_required_statuses(&collect_statuses(&bundle.case_truths), expected_path)?;
    Ok(())
}

fn validate_case_truth_contract(
    case_truth: &BamHaplogroupCaseTruth,
    expected_path: &Path,
) -> Result<()> {
    validate_case_truth_identity(case_truth, expected_path)?;
    validate_case_truth_summary(case_truth)?;
    validate_case_truth_status(case_truth)?;
    Ok(())
}

fn validate_case_truth_identity(
    case_truth: &BamHaplogroupCaseTruth,
    expected_path: &Path,
) -> Result<()> {
    if case_truth.case_id.trim().is_empty() || case_truth.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM haplogroup truth bundle `{}` contains a case with an empty case_id or sample_id",
            expected_path.display()
        ));
    }
    if case_truth.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must declare a non-empty cohort",
            case_truth.case_id
        ));
    }
    if case_truth.alignment_path.trim().is_empty()
        || case_truth.reference_panel_path.trim().is_empty()
    {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must declare non-empty input paths",
            case_truth.case_id
        ));
    }
    if case_truth.summary.summary_schema_version != BAM_HAPLOGROUP_TRUTH_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` uses schema `{}` instead of `{}`",
            case_truth.case_id,
            case_truth.summary.summary_schema_version,
            BAM_HAPLOGROUP_TRUTH_SUMMARY_SCHEMA_VERSION
        ));
    }
    if case_truth.summary.stage_id != "bam.haplogroups" {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must keep stage_id `bam.haplogroups`",
            case_truth.case_id
        ));
    }
    Ok(())
}

fn validate_case_truth_summary(case_truth: &BamHaplogroupCaseTruth) -> Result<()> {
    if case_truth.summary.method.trim().is_empty()
        || case_truth.summary.reference_panel_id.trim().is_empty()
        || case_truth.summary.reference_build.trim().is_empty()
        || case_truth.summary.population_scope.trim().is_empty()
        || case_truth.summary.status.trim().is_empty()
    {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must declare non-empty method, panel, build, population scope, and status",
            case_truth.case_id
        ));
    }
    if case_truth.summary.minimum_coverage < 0.0
        || case_truth.summary.observed_mean_coverage < 0.0
        || case_truth.summary.confidence < 0.0
        || case_truth.summary.confidence > 1.0
    {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must keep non-negative coverage values and confidence within [0, 1]",
            case_truth.case_id
        ));
    }
    if case_truth.summary.markers_total == 0
        || case_truth.summary.markers_supported > case_truth.summary.markers_total
    {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must keep marker counts consistent",
            case_truth.case_id
        ));
    }
    Ok(())
}

fn validate_case_truth_status(case_truth: &BamHaplogroupCaseTruth) -> Result<()> {
    match case_truth.summary.status.as_str() {
        "ready" => {
            if !case_truth.summary.ready
                || case_truth.summary.haplogroup_call.is_none()
                || !case_truth.summary.refusal_codes.is_empty()
            {
                return Err(anyhow!(
                    "BAM haplogroup truth case `{}` must keep a ready haplogroup call without refusal codes",
                    case_truth.case_id
                ));
            }
        }
        "uncertain_marker_support" => {
            if case_truth.summary.ready
                || case_truth.summary.haplogroup_call.is_none()
                || !case_truth
                    .summary
                    .refusal_codes
                    .iter()
                    .any(|code| code == "marker_support_incomplete")
            {
                return Err(anyhow!(
                    "BAM haplogroup truth case `{}` must keep partial marker support semantics",
                    case_truth.case_id
                ));
            }
        }
        "coverage_gate_not_met" => {
            if case_truth.summary.ready
                || case_truth.summary.haplogroup_call.is_some()
                || case_truth.summary.confidence != 0.0
                || !case_truth
                    .summary
                    .refusal_codes
                    .iter()
                    .any(|code| code == "coverage_below_haplogroup_minimum")
            {
                return Err(anyhow!(
                    "BAM haplogroup truth case `{}` must keep coverage-gated refusal semantics",
                    case_truth.case_id
                ));
            }
        }
        other => {
            return Err(anyhow!(
                "BAM haplogroup truth case `{}` has unsupported status `{other}`",
                case_truth.case_id
            ));
        }
    }
    Ok(())
}
