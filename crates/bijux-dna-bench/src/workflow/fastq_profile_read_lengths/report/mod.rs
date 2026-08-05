//! Owner: bijux-dna-bench
//! Source-locked report publication from admitted FASTQ read-length attempts.

mod evidence;
mod rendering;
mod science;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkCampaignPlan, BenchmarkStageCampaignSpec, BenchmarkStageReport,
};

use self::evidence::load_attempt_evidence;
pub use self::rendering::render_fastq_profile_read_lengths_markdown;
use super::super::{
    load_stage_campaign_registry, load_stage_interface_catalog_from_repo,
    verify_campaign_plan_source,
};
use crate::workflow::stage_report::{
    build_stage_report, canonical_directory, ensure_bytes, ensure_directory, is_active_plan_origin,
    require_complete_plan,
};
pub(super) use crate::workflow::stage_report::{hash_json, read_json, MAX_REPORT_INPUT_BYTES};

pub(super) const RUN_ID: &str = "run-2042";
pub(super) const STAGE_ID: &str = "fastq.profile_read_lengths";
pub(super) const BUNDLE_SCHEMA: &str = "bijux.benchmark.fastq.profile_read_lengths.bundle.v1";
pub(super) const METRICS_SCHEMA: &str = "bijux.benchmark.fastq.profile_read_lengths.metrics.v1";
const REPORT_FILE_STEM: &str = "run-2042-fastq-profile-read-lengths";

#[derive(Debug, Clone, PartialEq)]
pub struct FastqProfileReadLengthsStageReportPublication {
    pub report_path: PathBuf,
    pub markdown_path: PathBuf,
    pub report_sha256: String,
    pub markdown_sha256: String,
    pub report: BenchmarkStageReport,
}

/// Derive, validate, and no-clobber publish the stage report for admitted run-2042 attempts.
///
/// The report is accepted only when every planned tool attempt is passed, its admission can be
/// replayed against the complete output closure, scheduler accounting is terminal and successful,
/// and all tools share one source, input, parameter, and stage-interface identity.
///
/// # Errors
/// Returns an error when source authority, attempt closure, admitted evidence, scientific output,
/// comparability, or an existing publication is missing, malformed, stale, or conflicting.
pub fn publish_fastq_profile_read_lengths_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<FastqProfileReadLengthsStageReportPublication> {
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
    let repository_commit = bijux_dna_environment::api::repository_head_commit(&repository_root)?;
    if repository_commit != plan.repository_commit {
        bail!("benchmark report source worktree differs from campaign plan");
    }
    let registry = load_stage_campaign_registry(&registry_path)?;
    let campaign = registry
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == RUN_ID)
        .context("benchmark registry omits run-2042")?;
    require_governed_campaign(campaign, plan)?;
    let interfaces = load_stage_interface_catalog_from_repo(&interface_path, &repository_root)?;
    let interface = interfaces
        .interfaces
        .iter()
        .find(|interface| interface.stage_id == STAGE_ID)
        .context("benchmark stage interface omits FASTQ read lengths")?;

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

    ensure_directory(output_root)?;
    let report_path = output_root.join(format!("{REPORT_FILE_STEM}.json"));
    let markdown_path = output_root.join(format!("{REPORT_FILE_STEM}.md"));
    let mut report_bytes = serde_json::to_vec_pretty(&report)?;
    report_bytes.push(b'\n');
    let markdown = render_fastq_profile_read_lengths_markdown(&report);
    ensure_bytes(&report_path, &report_bytes)?;
    ensure_bytes(&markdown_path, markdown.as_bytes())?;
    Ok(FastqProfileReadLengthsStageReportPublication {
        report_path,
        markdown_path,
        report_sha256: bijux_dna_infra::hash_bytes_sha256(&report_bytes),
        markdown_sha256: bijux_dna_infra::hash_bytes_sha256(markdown.as_bytes()),
        report,
    })
}

fn require_governed_campaign(
    campaign: &BenchmarkStageCampaignSpec,
    plan: &BenchmarkCampaignPlan,
) -> Result<()> {
    if campaign.run_id != RUN_ID
        || campaign.stage_id != STAGE_ID
        || campaign.result_artifact != "read-lengths.json"
        || campaign.input_size_keys != ["fastq_input_bytes"]
        || campaign.measured_repetitions < 3
    {
        bail!("campaign is not the governed FASTQ read-length comparison");
    }
    require_complete_plan(campaign, plan)
}
