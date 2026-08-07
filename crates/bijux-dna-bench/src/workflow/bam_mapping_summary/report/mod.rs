//! Owner: bijux-dna-bench
//! Source-locked report publication from admitted BAM mapping-summary attempts.

mod evidence;
mod rendering;
mod science;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{BenchmarkCampaignPlan, BenchmarkStageCampaignSpec};

use self::evidence::load_attempt_evidence;
pub use self::rendering::render_bam_mapping_summary_markdown;
use super::super::{
    load_stage_campaign_registry, load_stage_interface_catalog_for_publication,
    verify_campaign_plan_source,
};
use crate::workflow::stage_report::{
    build_stage_report, canonical_directory, is_active_plan_origin, publish_stage_report,
    require_complete_plan, BenchmarkStageReportPublication,
};

pub(super) const RUN_ID: &str = "run-2004";
pub(super) const STAGE_ID: &str = "bam.mapping_summary";
pub type BamMappingSummaryStageReportPublication = BenchmarkStageReportPublication;

/// Derive, validate, and no-clobber publish the report for admitted run-2004 attempts.
///
/// # Errors
/// Returns an error when source authority, attempt closure, admitted evidence, scientific output,
/// comparability, or an existing publication is missing, malformed, stale, or conflicting.
pub fn publish_bam_mapping_summary_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamMappingSummaryStageReportPublication> {
    publish_bam_mapping_summary_stage_report_with_presentation_authority(
        repository_root,
        repository_root,
        study_root,
        benchmark_root,
        plan,
        output_root,
    )
}

pub(crate) fn publish_bam_mapping_summary_stage_report_with_presentation_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamMappingSummaryStageReportPublication> {
    let repository_root = canonical_directory(repository_root, "benchmark repository")?;
    let study_root = canonical_directory(study_root, "STUDY root")?;
    let benchmark_root = canonical_directory(benchmark_root, "benchmark result root")?;
    if !benchmark_root.starts_with(&study_root) {
        bail!("benchmark result root is outside STUDY");
    }
    let registry_path = repository_root.join("benchmarks/contracts/stage-campaigns.toml");
    let interface_path = repository_root.join("benchmarks/contracts/stage-interfaces.toml");
    let registry_sha256 = bijux_dna_infra::hash_file_sha256(&registry_path)?;
    verify_campaign_plan_source(plan, &plan.repository_commit, &registry_sha256)?;
    let repository_commit = bijux_dna_containers::api::repository_head_commit(&repository_root)?;
    if repository_commit != plan.repository_commit {
        bail!("benchmark report source worktree differs from campaign plan");
    }
    let registry = load_stage_campaign_registry(&registry_path)?;
    let campaign = registry
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == RUN_ID)
        .context("benchmark registry omits run-2004")?;
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
        .find(|interface| interface.stage_id == STAGE_ID)
        .context("benchmark stage interface omits BAM mapping summary")?;

    let mut evidence_by_tool = BTreeMap::new();
    for tool in &campaign.tool_ids {
        let planned = plan
            .cycle
            .attempts
            .iter()
            .filter(|attempt| is_active_plan_origin(attempt))
            .find(|attempt| attempt.identity.tool_id == *tool)
            .with_context(|| format!("campaign plan omits tool attempt {tool}"))?;
        let evidence = load_attempt_evidence(
            &repository_root,
            &study_root,
            &benchmark_root,
            campaign,
            interface,
            plan,
            &planned.identity,
        )?;
        evidence_by_tool.insert(tool.clone(), evidence);
    }
    let report = build_stage_report(&repository_root, plan, campaign, interface, evidence_by_tool)?;
    publish_stage_report(output_root, campaign, interface, report, &interfaces.presentation)
}

fn require_governed_campaign(
    campaign: &BenchmarkStageCampaignSpec,
    plan: &BenchmarkCampaignPlan,
) -> Result<()> {
    if campaign.run_id != RUN_ID
        || campaign.stage_id != STAGE_ID
        || campaign.tool_ids != ["picard", "samtools"]
        || campaign.result_artifact != "mapping-summary.tsv"
        || campaign.input_size_keys != ["bam_input_bytes", "bam_index_input_bytes"]
        || campaign.measured_repetitions < 3
    {
        bail!("campaign is not the governed BAM mapping-summary comparison");
    }
    require_complete_plan(campaign, plan)
}
