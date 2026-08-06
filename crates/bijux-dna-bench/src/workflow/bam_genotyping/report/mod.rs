//! Owner: bijux-dna-bench
//! Source-locked report publication from admitted BAM genotyping attempts.

mod evidence;
mod science;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkCampaignPlan, BenchmarkStageCampaignSpec, BenchmarkStageReport,
};

use self::evidence::load_attempt_evidence;
use super::super::bam_observation::render_bam_observation_markdown;
use super::super::{
    load_stage_campaign_registry, load_stage_interface_catalog_from_repo,
    verify_campaign_plan_source,
};
use crate::workflow::stage_report::{
    build_stage_report, canonical_directory, ensure_bytes, ensure_directory, is_active_plan_origin,
    require_complete_plan,
};

pub(super) const BUNDLE_SCHEMA: &str = "bijux.benchmark.bam.genotyping.bundle.v1";
const REPORT_FILE_STEM: &str = "run-2001-bam-genotyping";

#[derive(Debug, Clone, PartialEq)]
pub struct BamGenotypingStageReportPublication {
    pub report_path: PathBuf,
    pub markdown_path: PathBuf,
    pub report_sha256: String,
    pub markdown_sha256: String,
    pub report: BenchmarkStageReport,
}

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
        .find(|campaign| campaign.run_id == super::RUN_ID)
        .context("benchmark registry omits run-2001")?;
    require_governed_campaign(campaign, plan)?;
    let interfaces = load_stage_interface_catalog_from_repo(&interface_path, &repository_root)?;
    let interface = interfaces
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
    ensure_directory(output_root)?;
    let report_path = output_root.join(format!("{REPORT_FILE_STEM}.json"));
    let markdown_path = output_root.join(format!("{REPORT_FILE_STEM}.md"));
    let mut report_bytes = serde_json::to_vec_pretty(&report)?;
    report_bytes.push(b'\n');
    let markdown = render_bam_genotyping_markdown(&report);
    ensure_bytes(&report_path, &report_bytes)?;
    ensure_bytes(&markdown_path, markdown.as_bytes())?;
    Ok(BamGenotypingStageReportPublication {
        report_path,
        markdown_path,
        report_sha256: bijux_dna_infra::hash_bytes_sha256(&report_bytes),
        markdown_sha256: bijux_dna_infra::hash_bytes_sha256(markdown.as_bytes()),
        report,
    })
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
