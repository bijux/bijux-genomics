//! Owner: pure BAM campaign source admission and typed report derivation.
use crate::{
    validate_appraisal_report, validate_bam_benchmark_campaign_report,
    validate_benchmark_matrix_report, validate_hardening_queue_report, AppraisalFinding,
    AppraisalReport, BamBenchmarkCampaignReport, BamBenchmarkCampaignSummary,
    BamBenchmarkGoalEntry, BamMatrixRowRef, BamScenario, BenchmarkMatrixReport,
    HardeningQueueEntry, HardeningQueueReport, BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};
use anyhow::{anyhow, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

mod admission;
mod alignment;
mod ancient_dna;
mod assembly;
mod contamination;
mod filtering;
mod identity;
mod metrics;
mod resources;
mod validation;
use admission::selected_goal_ids;
use assembly::{build_goal_entries, summarize};

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
    rows: &'a [BamMatrixRowRef],
    findings: &'a [AppraisalFinding],
    queue_entries: &'a [HardeningQueueEntry],
}
static BAM_GOALS_CATALOG: LazyLock<Vec<GoalDefinition>> = LazyLock::new(|| {
    let mut goals: Vec<GoalDefinition> = [
        validation::GOALS,
        alignment::GOALS,
        filtering::GOALS,
        metrics::GOALS,
        ancient_dna::GOALS,
        contamination::GOALS,
        identity::GOALS,
        resources::GOALS,
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect();
    goals.sort_by_key(|goal| goal.goal_id);
    goals
});

/// Derive a validated BAM campaign from admitted matrix and appraisal products.
///
/// # Errors
/// Returns an error when a source product is invalid, campaign identities disagree, requested
/// goals are unknown, or the derived report violates its persisted contract.
pub fn derive_bam_benchmark_campaign(
    matrix: BenchmarkMatrixReport,
    appraisal: &AppraisalReport,
    queue: &HardeningQueueReport,
    requested_goals: Option<&str>,
) -> Result<BamBenchmarkCampaignReport> {
    validate_benchmark_matrix_report(&matrix).context("validate BAM campaign source matrix")?;
    validate_appraisal_report(appraisal).context("validate BAM campaign source appraisal")?;
    validate_hardening_queue_report(queue).context("validate BAM campaign source queue")?;
    if matrix.domain != "bam"
        || appraisal.domain != matrix.domain
        || queue.domain != matrix.domain
        || appraisal.campaign_id != matrix.campaign_id
        || queue.campaign_id != matrix.campaign_id
    {
        return Err(anyhow!("BAM campaign source identities do not align"));
    }
    let selected_goals = selected_goal_ids(requested_goals)?;
    let entries = build_goal_entries(&selected_goals, &matrix, &appraisal.findings, &queue.entries);
    let report = BamBenchmarkCampaignReport {
        schema_version: BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1.to_string(),
        campaign_id: matrix.campaign_id,
        domain: matrix.domain,
        selected_goals,
        summary: summarize(&entries),
        entries,
    };
    validate_bam_benchmark_campaign_report(&report)?;
    Ok(report)
}

#[cfg(test)]
#[path = "../../../tests/contracts/bam_campaign/mod.rs"]
mod tests;
