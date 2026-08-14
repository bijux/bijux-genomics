//! Owner: bijux-dna-bench
//! Source-locked report publication from admitted FASTQ profile attempts.

mod evidence;
mod rendering;
mod science;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{BenchmarkCampaignPlan, BenchmarkStageCampaignSpec};

use self::evidence::load_attempt_evidence;
pub use self::rendering::render_fastq_profile_reads_markdown;
use super::super::{
    load_stage_campaign_registry, load_stage_interface_catalog_for_publication,
    verify_campaign_plan_source_authority, BenchmarkCampaignPlanSource,
};
use crate::workflow::stage_report::{
    build_stage_report, canonical_directory, is_active_plan_origin, publish_stage_report,
    require_complete_plan, BenchmarkStageReportPublication,
};
pub(super) use crate::workflow::stage_report::{read_json, MAX_REPORT_INPUT_BYTES};

pub(super) const RUN_ID: &str = "run-2043";
pub(super) const STAGE_ID: &str = "fastq.profile_reads";
pub type FastqProfileReadsStageReportPublication = BenchmarkStageReportPublication;

/// Derive and no-clobber publish the run-2043 report from exact admitted attempt closure.
///
/// # Errors
/// Returns an error when source, plan, admission, scheduler, scientific, interface, or output
/// evidence is missing, stale, malformed, or conflicting.
pub fn publish_fastq_profile_reads_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<FastqProfileReadsStageReportPublication> {
    publish_fastq_profile_reads_stage_report_with_presentation_authority(
        repository_root,
        repository_root,
        study_root,
        benchmark_root,
        plan,
        BenchmarkCampaignPlanSource::Legacy,
        output_root,
    )
}

pub(crate) fn publish_fastq_profile_reads_stage_report_with_presentation_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    source_authority: BenchmarkCampaignPlanSource<'_>,
    output_root: &Path,
) -> Result<FastqProfileReadsStageReportPublication> {
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
        .find(|campaign| campaign.run_id == RUN_ID)
        .context("benchmark registry omits run-2043")?;
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
        .context("benchmark stage interface omits FASTQ profile reads")?;

    let mut evidence_by_tool = BTreeMap::new();
    for tool in &campaign.tool_ids {
        let planned = plan
            .cycle
            .attempts
            .iter()
            .filter(|attempt| is_active_plan_origin(attempt))
            .find(|attempt| attempt.identity.tool_id == *tool)
            .with_context(|| format!("campaign plan omits tool attempt {tool}"))?;
        evidence_by_tool.insert(
            tool.clone(),
            load_attempt_evidence(
                &repository_root,
                &study_root,
                &benchmark_root,
                campaign,
                interface,
                plan,
                &planned.identity,
            )?,
        );
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
        || campaign.result_artifact != "read-profile.tsv"
        || campaign.input_size_keys != ["fastq_input_bytes"]
        || campaign.measured_repetitions < 3
    {
        bail!("campaign is not the governed FASTQ profile comparison");
    }
    require_complete_plan(campaign, plan)
}
