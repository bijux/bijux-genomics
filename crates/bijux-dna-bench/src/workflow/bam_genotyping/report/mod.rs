//! Owner: bijux-dna-bench
//! Source-locked report publication from admitted BAM genotyping attempts.

mod evidence;
mod science;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkCampaignPlan, BenchmarkStageCampaignSpec, BenchmarkStageReport,
};

use self::evidence::load_attempt_evidence;
use super::super::bam_observation::render_bam_observation_markdown;
use super::super::{
    load_stage_campaign_registry, load_stage_interface_catalog_for_publication,
    verify_campaign_plan_source_authority, BenchmarkCampaignPlanSource,
};
use crate::workflow::stage_report::{
    build_stage_report, canonical_directory, is_active_plan_origin, publish_stage_report,
    require_complete_plan, BenchmarkStageReportPublication,
};

pub type BamGenotypingStageReportPublication = BenchmarkStageReportPublication;

/// Derive, validate, and no-clobber publish the report for admitted run-2001 attempts.
///
/// # Errors
/// Returns an error when source authority, attempt closure, admitted evidence, scientific output,
/// byte accounting, determinism, comparability, or publication integrity is invalid.
pub fn publish_bam_genotyping_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamGenotypingStageReportPublication> {
    publish_bam_genotyping_stage_report_with_presentation_authority(
        repository_root,
        repository_root,
        study_root,
        benchmark_root,
        plan,
        BenchmarkCampaignPlanSource::Legacy,
        output_root,
    )
}

pub(crate) fn publish_bam_genotyping_stage_report_with_presentation_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    source_authority: BenchmarkCampaignPlanSource<'_>,
    output_root: &Path,
) -> Result<BamGenotypingStageReportPublication> {
    let repository_root = canonical_directory(repository_root, "benchmark repository")?;
    let study_root = canonical_directory(study_root, "STUDY root")?;
    let benchmark_root = canonical_directory(benchmark_root, "benchmark result root")?;
    if !benchmark_root.starts_with(&study_root) {
        bail!("benchmark result root is outside STUDY");
    }
    let registry_path = repository_root.join("benchmarks/contracts/stage-campaigns.toml");
    let interface_path = repository_root.join("benchmarks/contracts/stage-interfaces.toml");
    let registry_sha256 = bijux_dna_infra::hash_file_sha256(&registry_path)?;
    verify_campaign_plan_source_authority(
        plan,
        source_authority,
        &plan.repository_commit,
        &registry_sha256,
    )?;
    let repository_commit = bijux_dna_containers::api::repository_head_commit(&repository_root)?;
    if repository_commit != plan.repository_commit {
        bail!("benchmark report source worktree differs from campaign plan");
    }
    let registry = load_stage_campaign_registry(&registry_path)?;
    let campaign = registry
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == super::RUN_ID)
        .context("benchmark registry omits run-2001")?;
    require_governed_campaign(campaign, plan)?;
    let interfaces = load_stage_interface_catalog_for_publication(
        &interface_path,
        &repository_root,
        presentation_repository_root,
    )?;
    let interface = interfaces
        .catalog
        .interfaces
        .iter()
        .find(|interface| interface.stage_id == super::STAGE_ID)
        .context("benchmark stage interface omits BAM genotyping")?;

    let planned = plan
        .cycle
        .attempts
        .iter()
        .filter(|attempt| is_active_plan_origin(attempt))
        .find(|attempt| attempt.identity.tool_id == "bcftools")
        .context("campaign plan omits bcftools attempt")?;
    let evidence = load_attempt_evidence(
        &repository_root,
        &study_root,
        &benchmark_root,
        campaign,
        interface,
        plan,
        &planned.identity,
    )?;
    let report = build_stage_report(
        &repository_root,
        plan,
        campaign,
        interface,
        BTreeMap::from([("bcftools".to_string(), evidence)]),
    )?;
    publish_stage_report(output_root, campaign, interface, report, &interfaces.presentation)
}

#[must_use]
pub fn render_bam_genotyping_markdown(report: &BenchmarkStageReport) -> String {
    render_bam_observation_markdown(report)
}

fn require_governed_campaign(
    campaign: &BenchmarkStageCampaignSpec,
    plan: &BenchmarkCampaignPlan,
) -> Result<()> {
    if campaign.run_id != super::RUN_ID
        || campaign.stage_id != super::STAGE_ID
        || campaign.tool_ids != ["bcftools"]
        || campaign.result_artifact != "genotypes.tsv"
        || campaign.input_size_keys
            != ["bam_input_bytes", "reference_input_bytes", "database_input_bytes"]
        || campaign.measured_repetitions < 3
    {
        bail!("campaign is not the governed BAM genotyping qualification");
    }
    require_complete_plan(campaign, plan)
}
