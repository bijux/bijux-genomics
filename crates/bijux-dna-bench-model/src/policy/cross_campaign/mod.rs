//! Owner: pure cross-domain campaign admission and typed evidence projection.
use crate::{
    validate_appraisal_report, validate_benchmark_matrix_report,
    validate_cross_benchmark_campaign_report, validate_hardening_queue_report, AppraisalFinding,
    AppraisalReport, BenchmarkMatrixReport, CrossBenchmarkCampaignReport,
    CrossBenchmarkCampaignSummary, CrossBenchmarkGoalEntry, CrossMatrixRowRef, CrossScenario,
    HardeningQueueEntry, HardeningQueueReport, CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};
use anyhow::{anyhow, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

mod admission;
mod assembly;
mod domain_handoffs;
mod execution_policy;
mod reviewer_evidence;
mod source_integrity;
use admission::selected_goal_ids;
use assembly::{build_goal_entries, summarize};

const BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN: &str = "fastq.trim_reads=>bam.align";
const BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN: &str = "fastq.validate_reads=>bam.align";
const BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE: &str = "fastq.trim_terminal_damage=>bam.damage";
const BRIDGE_BAM_GENOTYPING_TO_VCF_CALL: &str = "bam.genotyping=>vcf.call";
const BRIDGE_FASTQ_TRIM_TO_VCF_GL: &str = "fastq.trim_reads=>vcf.call_gl";
const BRIDGE_BAM_SUMMARY_TO_VCF_STATS: &str = "bam.mapping_summary=>vcf.stats";
const BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY: &str = "fastq.profile_reads=>bam.mapping_summary";
const BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION: &str = "fastq.validate_reads=>bam.contamination";
const BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER: &str = "bam.contamination=>vcf.filter";
const BRIDGE_VCF_FILTER_TO_VCF_STATS: &str = "vcf.filter=>vcf.stats";

#[derive(Clone, Copy)]
struct GoalDefinition {
    goal_id: &'static str,
    title: &'static str,
    stage_ids: &'static [&'static str],
    scenarios: &'static [ScenarioDefinition],
    checks: fn(&CheckInputs<'_>) -> Vec<String>,
}

struct ScenarioDefinition {
    scenario_id: &'static str,
    focus: &'static str,
}

struct CheckInputs<'a> {
    rows: &'a [CrossMatrixRowRef],
    queue_entries: &'a [HardeningQueueEntry],
}

static CROSS_GOALS_CATALOG: LazyLock<Vec<GoalDefinition>> = LazyLock::new(|| {
    let mut goals: Vec<GoalDefinition> = [
        domain_handoffs::GOALS,
        execution_policy::GOALS,
        source_integrity::GOALS,
        reviewer_evidence::GOALS,
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect();
    goals.sort_by_key(|goal| goal.goal_id);
    goals
});

/// Derive a validated cross-domain campaign from admitted matrix and appraisal products.
///
/// # Errors
/// Returns an error when a source product is invalid, campaign identities disagree, requested
/// goals are unknown, or the derived report violates its persisted contract.
pub fn derive_cross_benchmark_campaign(
    matrix: BenchmarkMatrixReport,
    appraisal: &AppraisalReport,
    queue: &HardeningQueueReport,
    requested_goals: Option<&str>,
) -> Result<CrossBenchmarkCampaignReport> {
    validate_benchmark_matrix_report(&matrix).context("validate cross-campaign source matrix")?;
    validate_appraisal_report(appraisal).context("validate cross-campaign source appraisal")?;
    validate_hardening_queue_report(queue).context("validate cross-campaign source queue")?;
    if matrix.domain != "cross"
        || appraisal.domain != matrix.domain
        || queue.domain != matrix.domain
        || appraisal.campaign_id != matrix.campaign_id
        || queue.campaign_id != matrix.campaign_id
    {
        return Err(anyhow!("cross-campaign source identities do not align"));
    }
    let selected_goals = selected_goal_ids(requested_goals)?;
    let entries = build_goal_entries(&selected_goals, &matrix, &appraisal.findings, &queue.entries);
    let report = CrossBenchmarkCampaignReport {
        schema_version: CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1.to_string(),
        campaign_id: matrix.campaign_id,
        domain: matrix.domain,
        selected_goals,
        summary: summarize(&entries),
        entries,
    };
    validate_cross_benchmark_campaign_report(&report)?;
    Ok(report)
}

#[cfg(test)]
#[path = "../../../tests/contracts/cross_campaign/mod.rs"]
mod tests;
