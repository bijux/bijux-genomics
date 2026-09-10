//! Owner: pure hardening campaign admission, source joins and goal projections.
use crate::{
    validate_appraisal_report, validate_benchmark_matrix_report,
    validate_hardening_benchmark_campaign_report, validate_hardening_queue_report,
    AppraisalFinding, AppraisalReport, BenchmarkMatrixReport, HardeningBenchmarkCampaignReport,
    HardeningBenchmarkCampaignSummary, HardeningBenchmarkGoalEntry, HardeningMatrixRowRef,
    HardeningQueueEntry, HardeningQueueReport, HardeningScenario,
    HARDENING_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};
use anyhow::{anyhow, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

mod comparisons;
mod corpora;
mod databases;
mod diagnostics;
mod evidence;
mod execution;
mod operations;
mod planning;
mod security;
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
    rows: &'a [HardeningMatrixRowRef],
    findings: &'a [AppraisalFinding],
    queue_entries: &'a [HardeningQueueEntry],
}
static HARDENING_GOALS_CATALOG: LazyLock<Vec<GoalDefinition>> = LazyLock::new(|| {
    let mut goals: Vec<GoalDefinition> = [
        diagnostics::GOALS,
        execution::GOALS,
        corpora::GOALS,
        databases::GOALS,
        planning::GOALS,
        comparisons::GOALS,
        evidence::GOALS,
        security::GOALS,
        operations::GOALS,
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect();
    goals.sort_by_key(|goal| goal.goal_id);
    goals
});
fn selected_goal_ids(raw: Option<&str>) -> Result<Vec<String>> {
    let available = HARDENING_GOALS_CATALOG
        .iter()
        .map(|goal| goal.goal_id.to_string())
        .collect::<BTreeSet<_>>();
    let mut selected = if let Some(value) = raw {
        value
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(str::to_ascii_uppercase)
            .collect::<BTreeSet<_>>()
    } else {
        available.clone()
    };
    if selected.is_empty() {
        return Err(anyhow!("hardening-benchmark-campaign requires at least one goal"));
    }
    let unknown =
        selected.iter().filter(|goal| !available.contains(*goal)).cloned().collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(anyhow!("unknown hardening goals requested: {}", unknown.join(",")));
    }
    let mut ordered = HARDENING_GOALS_CATALOG
        .iter()
        .filter_map(|goal| {
            if selected.remove(goal.goal_id) {
                Some(goal.goal_id.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    ordered.sort();
    Ok(ordered)
}

fn status_for_goal(
    rows: &[HardeningMatrixRowRef],
    queue_entries: &[HardeningQueueEntry],
) -> String {
    if rows.is_empty() {
        return "missing-stage-binding".to_string();
    }
    if rows.iter().any(|row| row.readiness_class == "refuse") {
        return "requires-hardening".to_string();
    }
    if !queue_entries.is_empty() {
        return "requires-hardening".to_string();
    }
    "ready-for-benchmark-run".to_string()
}

fn build_goal_entries(
    selected_goal_ids: &[String],
    matrix: &BenchmarkMatrixReport,
    findings: &[AppraisalFinding],
    queue: &[HardeningQueueEntry],
) -> Vec<HardeningBenchmarkGoalEntry> {
    let selected = selected_goal_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut entries = Vec::new();

    for goal in HARDENING_GOALS_CATALOG.iter().filter(|goal| selected.contains(goal.goal_id)) {
        let stage_set = goal.stage_ids.iter().copied().collect::<BTreeSet<_>>();
        let matched_rows = matrix
            .rows
            .iter()
            .filter(|row| stage_set.contains(row.stage_id.as_str()))
            .map(|row| HardeningMatrixRowRef {
                row_id: row.row_id.clone(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                readiness_class: row.readiness.class.clone(),
                repetitions: row.repetitions,
            })
            .collect::<Vec<_>>();

        let row_ids = matched_rows.iter().map(|row| row.row_id.clone()).collect::<BTreeSet<_>>();

        let appraisal_findings = findings
            .iter()
            .filter(|finding| row_ids.contains(&finding.row_id))
            .cloned()
            .collect::<Vec<_>>();

        let hardening_entries = queue
            .iter()
            .filter_map(|entry| {
                let mut projected = entry.clone();
                projected.affected_rows.retain(|row| row_ids.contains(row));
                (!projected.affected_rows.is_empty()).then_some(projected)
            })
            .collect::<Vec<_>>();

        let status = status_for_goal(&matched_rows, &hardening_entries);
        let goal_checks = (goal.checks)(&CheckInputs {
            rows: &matched_rows,
            findings: &appraisal_findings,
            queue_entries: &hardening_entries,
        });

        entries.push(HardeningBenchmarkGoalEntry {
            goal_id: goal.goal_id.to_string(),
            title: goal.title.to_string(),
            stage_ids: goal.stage_ids.iter().map(|stage| (*stage).to_string()).collect(),
            scenarios: goal
                .scenarios
                .iter()
                .map(|scenario| HardeningScenario {
                    scenario_id: scenario.scenario_id.to_string(),
                    focus: scenario.focus.to_string(),
                })
                .collect(),
            matched_rows,
            appraisal_findings,
            hardening_entries,
            goal_checks,
            status,
        });
    }

    entries
}

fn summarize(entries: &[HardeningBenchmarkGoalEntry]) -> HardeningBenchmarkCampaignSummary {
    let mut status_counts = BTreeMap::new();
    let mut total_rows = 0;
    let mut total_findings = 0;
    let mut total_queue_entries = 0;
    for entry in entries {
        *status_counts.entry(entry.status.clone()).or_insert(0) += 1;
        total_rows += entry.matched_rows.len();
        total_findings += entry.appraisal_findings.len();
        total_queue_entries += entry.hardening_entries.len();
    }
    HardeningBenchmarkCampaignSummary {
        total_goals: entries.len(),
        total_rows,
        total_findings,
        total_queue_entries,
        status_counts,
    }
}

/// Derive a validated hardening campaign from admitted matrix and appraisal products.
///
/// # Errors
/// Returns an error when a source product is invalid, campaign identities disagree, requested
/// goals are unknown, or the derived report violates its persisted contract.
pub fn derive_hardening_benchmark_campaign(
    matrix: BenchmarkMatrixReport,
    appraisal: &AppraisalReport,
    queue: &HardeningQueueReport,
    requested_goals: Option<&str>,
) -> Result<HardeningBenchmarkCampaignReport> {
    validate_benchmark_matrix_report(&matrix)
        .context("validate hardening campaign source matrix")?;
    validate_appraisal_report(appraisal).context("validate hardening campaign source appraisal")?;
    validate_hardening_queue_report(queue).context("validate hardening campaign source queue")?;
    if matrix.domain != "all"
        || appraisal.domain != matrix.domain
        || queue.domain != matrix.domain
        || appraisal.campaign_id != matrix.campaign_id
        || queue.campaign_id != matrix.campaign_id
    {
        return Err(anyhow!("hardening campaign source identities do not align"));
    }
    let selected_goals = selected_goal_ids(requested_goals)?;
    let entries = build_goal_entries(&selected_goals, &matrix, &appraisal.findings, &queue.entries);
    let report = HardeningBenchmarkCampaignReport {
        schema_version: HARDENING_BENCHMARK_CAMPAIGN_SCHEMA_V1.to_string(),
        campaign_id: matrix.campaign_id,
        domain: matrix.domain,
        selected_goals,
        summary: summarize(&entries),
        entries,
    };
    validate_hardening_benchmark_campaign_report(&report)?;
    Ok(report)
}

#[cfg(test)]
mod tests;
