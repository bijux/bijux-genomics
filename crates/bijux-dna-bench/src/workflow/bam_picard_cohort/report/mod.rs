//! Owner: bijux-dna-bench
//! Source-locked publication for stratified Picard BAM cohort campaigns.

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

pub type BamPicardCohortStageReportPublication = BenchmarkStageReportPublication;

struct ReportPolicy {
    run_id: &'static str,
    stage_id: &'static str,
    result_stem: &'static str,
    scientific_metrics: &'static [&'static str],
    requires_reference: bool,
}

const GC_BIAS_POLICY: ReportPolicy = ReportPolicy {
    run_id: "run-2013",
    stage_id: "bam.gc_bias",
    result_stem: "gc-bias-summary",
    scientific_metrics: super::GC_BIAS_METRICS,
    requires_reference: true,
};
const INSERT_SIZE_POLICY: ReportPolicy = ReportPolicy {
    run_id: "run-2015",
    stage_id: "bam.insert_size",
    result_stem: "insert-size-summary",
    scientific_metrics: super::INSERT_SIZE_METRICS,
    requires_reference: false,
};

pub fn publish_bam_gc_bias_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamPicardCohortStageReportPublication> {
    publish_bam_gc_bias_stage_report_with_presentation_authority(
        repository_root,
        repository_root,
        study_root,
        benchmark_root,
        plan,
        output_root,
    )
}

pub(crate) fn publish_bam_gc_bias_stage_report_with_presentation_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamPicardCohortStageReportPublication> {
    publish(
        repository_root,
        presentation_repository_root,
        study_root,
        benchmark_root,
        plan,
        output_root,
        &GC_BIAS_POLICY,
    )
}

pub fn publish_bam_insert_size_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamPicardCohortStageReportPublication> {
    publish_bam_insert_size_stage_report_with_presentation_authority(
        repository_root,
        repository_root,
        study_root,
        benchmark_root,
        plan,
        output_root,
    )
}

pub(crate) fn publish_bam_insert_size_stage_report_with_presentation_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BamPicardCohortStageReportPublication> {
    publish(
        repository_root,
        presentation_repository_root,
        study_root,
        benchmark_root,
        plan,
        output_root,
        &INSERT_SIZE_POLICY,
    )
}

#[allow(clippy::too_many_arguments)]
fn publish(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
    policy: &ReportPolicy,
) -> Result<BamPicardCohortStageReportPublication> {
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
        .find(|campaign| campaign.run_id == policy.run_id)
        .with_context(|| format!("benchmark registry omits {}", policy.run_id))?;
    require_campaign(campaign, plan, policy)?;
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
        .with_context(|| format!("benchmark interface omits {}", policy.stage_id))?;
    let planned = plan
        .cycle
        .attempts
        .iter()
        .filter(|attempt| is_active_plan_origin(attempt))
        .find(|attempt| attempt.identity.tool_id == "picard")
        .context("Picard cohort plan omits its active attempt")?;
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
    let report = build_stage_report(
        &repository_root,
        plan,
        campaign,
        interface,
        BTreeMap::from([("picard".to_string(), evidence)]),
    )?;
    publish_stage_report(output_root, campaign, interface, report, &interfaces.presentation)
}

fn require_campaign(
    campaign: &BenchmarkStageCampaignSpec,
    plan: &BenchmarkCampaignPlan,
    policy: &ReportPolicy,
) -> Result<()> {
    let expected_input_keys = if policy.requires_reference {
        &["bam_input_bytes", "bam_index_input_bytes", "reference_input_bytes"][..]
    } else {
        &["bam_input_bytes", "bam_index_input_bytes"][..]
    };
    if campaign.run_id != policy.run_id
        || campaign.stage_id != policy.stage_id
        || campaign.tool_ids != ["picard"]
        || campaign.corpus_ids != [super::CORPUS_ID]
        || campaign.scientific_metrics.iter().map(String::as_str).collect::<Vec<_>>()
            != policy.scientific_metrics
        || campaign.input_size_keys.iter().map(String::as_str).collect::<Vec<_>>()
            != expected_input_keys
        || campaign.result_artifact != format!("{}.tsv", policy.result_stem)
        || campaign.measured_repetitions < 3
    {
        bail!("campaign differs from its governed Picard BAM cohort policy");
    }
    require_complete_plan(campaign, plan)
}
