//! Owner: bijux-dna-bench
//! Source-locked publication for the controlled BAM alignment comparison.

mod evidence;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{BenchmarkCampaignPlan, BenchmarkStageCampaignSpec};

use self::evidence::load_attempt_evidence;
use super::super::{
    load_stage_campaign_registry, load_stage_interface_catalog_for_publication,
    verify_campaign_plan_source,
};
use crate::workflow::stage_report::{
    build_stage_report, canonical_directory, is_active_plan_origin, publish_stage_report,
    require_complete_plan, BenchmarkStageReportPublication,
};

pub type BamAlignmentStageReportPublication = BenchmarkStageReportPublication;

pub fn publish_bam_alignment_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamAlignmentStageReportPublication> {
    publish_bam_alignment_stage_report_with_presentation_authority(
        repository_root,
        repository_root,
        study_root,
        benchmark_root,
        plan,
        output_root,
    )
}

pub(crate) fn publish_bam_alignment_stage_report_with_presentation_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamAlignmentStageReportPublication> {
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
    if bijux_dna_containers::api::repository_head_commit(&repository_root)?
        != plan.repository_commit
    {
        bail!("benchmark report source worktree differs from campaign plan");
    }
    let registry = load_stage_campaign_registry(&registry_path)?;
    let campaign = registry
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == super::RUN_ID)
        .context("benchmark registry omits run-2005")?;
    require_campaign(campaign, plan)?;
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
        .context("benchmark interface omits bam.align")?;
    let evidence = plan
        .cycle
        .attempts
        .iter()
        .filter(|attempt| is_active_plan_origin(attempt))
        .map(|attempt| {
            Ok((
                attempt.identity.tool_id.clone(),
                load_attempt_evidence(
                    &repository_root,
                    &study_root,
                    &benchmark_root,
                    campaign,
                    interface,
                    plan,
                    &attempt.identity,
                )?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let report = build_stage_report(&repository_root, plan, campaign, interface, evidence)?;
    publish_stage_report(output_root, campaign, interface, report, &interfaces.presentation)
}

fn require_campaign(
    campaign: &BenchmarkStageCampaignSpec,
    plan: &BenchmarkCampaignPlan,
) -> Result<()> {
    let scientific_metrics = [
        "total_reads",
        "mapped_reads",
        "alignment_rate",
        "expected_mapped_reads",
        "exact_placement_reads",
        "placement_precision",
        "placement_recall",
    ];
    if campaign.run_id != super::RUN_ID
        || campaign.stage_id != super::STAGE_ID
        || campaign.tool_ids != ["bowtie2", "bwa"]
        || campaign.corpus_ids != [super::PANEL_ID]
        || campaign.reference_catalog_ids != [super::REFERENCE_ID]
        || campaign.scientific_metrics.iter().map(String::as_str).collect::<Vec<_>>()
            != scientific_metrics
        || campaign.result_artifact != "alignment-truth-summary.tsv"
        || campaign.measured_repetitions < 3
    {
        bail!("campaign differs from the governed run-2005 alignment policy");
    }
    require_complete_plan(campaign, plan)
}
