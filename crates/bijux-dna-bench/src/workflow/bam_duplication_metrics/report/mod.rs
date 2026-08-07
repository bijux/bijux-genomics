//! Owner: bijux-dna-bench
//! Source-locked report publication for the governed BAM duplication-metrics campaign.

mod science;

use std::path::Path;

use anyhow::Result;
use bijux_dna_bench_model::{BenchmarkCampaignPlan, BenchmarkStageReport};

use self::science::derive_scientific_observation;
use super::super::bam_observation::{
    publish_bam_observation_stage_report, render_bam_observation_markdown,
    BamObservationReportPolicy, BamObservationStageReportPublication,
};

pub type BamDuplicationMetricsStageReportPublication = BamObservationStageReportPublication;

const POLICY: BamObservationReportPolicy = BamObservationReportPolicy {
    run_id: "run-2011",
    stage_id: "bam.duplication_metrics",
    tool_ids: &["picard", "samtools"],
    result_artifact: "duplication-summary.tsv",
    input_size_keys: &["bam_input_bytes", "bam_index_input_bytes"],
    digest_basis: "normalized duplication-summary TSV bytes",
    derive: derive_scientific_observation,
};

/// Derive and no-clobber publish the report for admitted run-2011 attempts.
///
/// # Errors
/// Returns an error when source authority, admitted evidence, output closure, or scientific
/// consistency is missing, stale, malformed, or conflicting.
pub fn publish_bam_duplication_metrics_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamDuplicationMetricsStageReportPublication> {
    publish_bam_duplication_metrics_stage_report_with_presentation_authority(
        repository_root,
        repository_root,
        study_root,
        benchmark_root,
        plan,
        output_root,
    )
}

pub(crate) fn publish_bam_duplication_metrics_stage_report_with_presentation_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamDuplicationMetricsStageReportPublication> {
    publish_bam_observation_stage_report(
        repository_root,
        presentation_repository_root,
        study_root,
        benchmark_root,
        plan,
        output_root,
        &POLICY,
    )
}

#[must_use]
pub fn render_bam_duplication_metrics_markdown(report: &BenchmarkStageReport) -> String {
    render_bam_observation_markdown(report)
}
