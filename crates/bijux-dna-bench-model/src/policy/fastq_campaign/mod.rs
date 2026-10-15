//! Owner: pure FASTQ campaign admission and typed evidence projection.
use crate::{
    validate_appraisal_report, validate_benchmark_matrix_report,
    validate_fastq_benchmark_campaign_report, validate_hardening_queue_report, AppraisalFinding,
    AppraisalReport, BenchmarkMatrixReport, FastqBenchmarkCampaignReport,
    FastqBenchmarkCampaignSummary, FastqBenchmarkGoalEntry, FastqMatrixRowRef, FastqScenario,
    HardeningQueueEntry, HardeningQueueReport, FASTQ_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};
use anyhow::{anyhow, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

mod admission;
mod assembly;
mod depletion;
mod evidence;
mod preprocessing;
mod read_layout;
mod sampling;
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
    rows: &'a [FastqMatrixRowRef],
    findings: &'a [AppraisalFinding],
    queue_entries: &'a [HardeningQueueEntry],
}

static FASTQ_GOALS_CATALOG: LazyLock<Vec<GoalDefinition>> = LazyLock::new(|| {
    let mut goals: Vec<GoalDefinition> = [
        validation::GOALS,
        read_layout::GOALS,
        sampling::GOALS,
        preprocessing::GOALS,
        depletion::GOALS,
        evidence::GOALS,
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect();
    goals.sort_by_key(|goal| goal.goal_id);
    goals
});

/// Derive a validated FASTQ campaign from admitted matrix and appraisal products.
///
/// # Errors
/// Returns an error when a source product is invalid, campaign identities disagree, requested
/// goals are unknown, or the derived report violates its persisted contract.
pub fn derive_fastq_benchmark_campaign(
    matrix: BenchmarkMatrixReport,
    appraisal: &AppraisalReport,
    queue: &HardeningQueueReport,
    requested_goals: Option<&str>,
) -> Result<FastqBenchmarkCampaignReport> {
    validate_benchmark_matrix_report(&matrix).context("validate FASTQ campaign source matrix")?;
    validate_appraisal_report(appraisal).context("validate FASTQ campaign source appraisal")?;
    validate_hardening_queue_report(queue).context("validate FASTQ campaign source queue")?;
    if matrix.domain != "fastq"
        || appraisal.domain != matrix.domain
        || queue.domain != matrix.domain
        || appraisal.campaign_id != matrix.campaign_id
        || queue.campaign_id != matrix.campaign_id
    {
        return Err(anyhow!("FASTQ campaign source identities do not align"));
    }
    let selected_goals = selected_goal_ids(requested_goals)?;
    let entries = build_goal_entries(&selected_goals, &matrix, &appraisal.findings, &queue.entries);
    let report = FastqBenchmarkCampaignReport {
        schema_version: FASTQ_BENCHMARK_CAMPAIGN_SCHEMA_V1.to_string(),
        campaign_id: matrix.campaign_id,
        domain: matrix.domain,
        selected_goals,
        summary: summarize(&entries),
        entries,
    };
    validate_fastq_benchmark_campaign_report(&report)?;
    Ok(report)
}

#[cfg(test)]
#[path = "../../../tests/contracts/fastq_campaign/mod.rs"]
mod tests;
