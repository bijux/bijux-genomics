//! Owner: bijux-dna-bench
//! Shared source-locked publication for BAM observational stage reports.

mod evidence;
mod rendering;
mod science;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkCampaignPlan, BenchmarkExecutionSpec, BenchmarkStageCampaignSpec,
};

use self::evidence::load_attempt_evidence;
pub(crate) use self::rendering::render_bam_observation_markdown;
pub(crate) use self::science::{
    numeric_value, read_numeric_summary, validate_byte_counts, ScientificObservation,
};
use super::super::{
    load_stage_campaign_registry, load_stage_interface_catalog_for_publication,
    verify_campaign_plan_source_authority, BenchmarkCampaignPlanSource,
};
use crate::workflow::stage_report::{
    build_stage_report, canonical_directory, is_active_plan_origin, publish_stage_report,
    require_complete_plan, BenchmarkStageReportPublication,
};

pub(crate) type ScientificDeriver =
    fn(&BenchmarkExecutionSpec, u32) -> Result<ScientificObservation>;

pub(crate) struct BamObservationReportPolicy {
    pub(crate) run_id: &'static str,
    pub(crate) stage_id: &'static str,
    pub(crate) tool_ids: &'static [&'static str],
    pub(crate) result_artifact: &'static str,
    pub(crate) input_size_keys: &'static [&'static str],
    pub(crate) digest_basis: &'static str,
    pub(crate) derive: ScientificDeriver,
}

pub type BamObservationStageReportPublication = BenchmarkStageReportPublication;

#[allow(clippy::too_many_arguments)]
pub(crate) fn publish_bam_observation_stage_report(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    source_authority: BenchmarkCampaignPlanSource<'_>,
    output_root: &Path,
    policy: &BamObservationReportPolicy,
) -> Result<BamObservationStageReportPublication> {
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
        .find(|campaign| campaign.run_id == policy.run_id)
        .with_context(|| format!("benchmark registry omits {}", policy.run_id))?;
    require_governed_campaign(campaign, plan, policy)?;
    let interfaces = load_stage_interface_catalog_for_publication(
        &interface_path,
        &repository_root,
        presentation_repository_root,
    )?;
    let interface = interfaces
        .catalog
        .interfaces
        .iter()
        .find(|interface| interface.stage_id == policy.stage_id)
        .with_context(|| format!("benchmark stage interface omits {}", policy.stage_id))?;

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
            policy,
        )?;
        evidence_by_tool.insert(tool.clone(), evidence);
    }
    let report = build_stage_report(&repository_root, plan, campaign, interface, evidence_by_tool)?;
    publish_stage_report(output_root, campaign, interface, report, &interfaces.presentation)
}

fn require_governed_campaign(
    campaign: &BenchmarkStageCampaignSpec,
    plan: &BenchmarkCampaignPlan,
    policy: &BamObservationReportPolicy,
) -> Result<()> {
    let tools = campaign.tool_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let input_keys = campaign.input_size_keys.iter().map(String::as_str).collect::<Vec<_>>();
    if campaign.run_id != policy.run_id
        || campaign.stage_id != policy.stage_id
        || tools != policy.tool_ids
        || campaign.result_artifact != policy.result_artifact
        || input_keys != policy.input_size_keys
        || campaign.measured_repetitions < 3
    {
        bail!("campaign differs from its governed BAM observational report policy");
    }
    require_complete_plan(campaign, plan)
}
