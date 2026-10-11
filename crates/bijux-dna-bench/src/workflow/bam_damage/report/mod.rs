//! Owner: bijux-dna-bench
//! Source-locked report publication for the governed BAM damage qualification campaign.

mod science;

use std::path::Path;

use anyhow::Result;
use bijux_dna_bench_model::{BenchmarkCampaignPlan, BenchmarkStageReport};

use self::science::derive_scientific_observation;
use super::super::bam_observation::{
    publish_bam_observation_stage_report, render_bam_observation_markdown,
    BamObservationReportPolicy, BamObservationStageReportPublication,
};
use crate::workflow::BenchmarkCampaignPlanSource;

pub type BamDamageStageReportPublication = BamObservationStageReportPublication;

const POLICY: BamObservationReportPolicy = BamObservationReportPolicy {
    run_id: super::RUN_ID,
    stage_id: super::STAGE_ID,
    tool_ids: &["addeam", "damageprofiler", "mapdamage2", "ngsbriggs", "pmdtools", "pydamage"],
    result_artifact: "damage-summary.tsv",
    input_size_keys: &["bam_input_bytes", "bam_index_input_bytes", "reference_input_bytes"],
    digest_basis: "normalized damage-summary TSV bytes",
    derive: derive_scientific_observation,
};

/// Derive and no-clobber publish the report for admitted run-2010 attempts.
///
/// This qualification preserves all six supported tools and explicitly carries the licensed
/// ngsBriggs implementation through the same evidence and publication path.
///
/// # Errors
/// Returns an error when source authority, admitted evidence, native output closure, normalized
/// values, reference-inclusive byte accounting, or downstream probe semantics are invalid.
pub fn publish_bam_damage_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamDamageStageReportPublication> {
    publish_bam_damage_stage_report_with_presentation_authority(
        repository_root,
        repository_root,
        study_root,
        benchmark_root,
        plan,
        BenchmarkCampaignPlanSource::Legacy,
        output_root,
    )
}

pub(crate) fn publish_bam_damage_stage_report_with_presentation_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    source_authority: BenchmarkCampaignPlanSource<'_>,
    output_root: &Path,
) -> Result<BamDamageStageReportPublication> {
    publish_bam_observation_stage_report(
        repository_root,
        presentation_repository_root,
        study_root,
        benchmark_root,
        plan,
        source_authority,
        output_root,
        &POLICY,
    )
}

#[must_use]
pub fn render_bam_damage_markdown(report: &BenchmarkStageReport) -> String {
    render_bam_observation_markdown(report)
}
