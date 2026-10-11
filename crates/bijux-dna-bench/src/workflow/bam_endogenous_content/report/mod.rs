//! Owner: bijux-dna-bench
//! Source-locked report publication for the governed BAM endogenous-content campaign.

mod science;

use std::path::Path;

use anyhow::Result;
use bijux_dna_bench_model::{BenchmarkCampaignPlan, BenchmarkStageReport};

use self::science::derive_scientific_observation;
use super::super::bam_observation::{
    publish_bam_observation_stage_report, render_bam_observation_markdown,
    BamObservationReportPolicy, BamObservationStageReportPublication,
};

pub type BamEndogenousContentStageReportPublication = BamObservationStageReportPublication;

const POLICY: BamObservationReportPolicy = BamObservationReportPolicy {
    run_id: "run-2003",
    stage_id: "bam.endogenous_content",
    report_file_stem: "run-2003-bam-endogenous-content",
    tool_ids: &["samtools"],
    result_artifact: "endogenous-content.tsv",
    input_size_keys: &["bam_input_bytes", "bam_index_input_bytes"],
    digest_basis: "normalized endogenous-content TSV bytes",
    derive: derive_scientific_observation,
};

/// Derive and no-clobber publish the report for the admitted run-2003 attempt.
///
/// # Errors
/// Returns an error when source authority, admitted evidence, output closure, or scientific
/// consistency is missing, stale, malformed, or conflicting.
pub fn publish_bam_endogenous_content_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamEndogenousContentStageReportPublication> {
    publish_bam_observation_stage_report(
        repository_root,
        study_root,
        benchmark_root,
        plan,
        output_root,
        &POLICY,
    )
}

#[must_use]
pub fn render_bam_endogenous_content_markdown(report: &BenchmarkStageReport) -> String {
    render_bam_observation_markdown(report)
}
