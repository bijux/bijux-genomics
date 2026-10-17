//! Owner: pure BAM campaign metrics goal declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};

const MAPPING_SUMMARY_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "mapped-unmapped-balance",
        focus: "mapped/unmapped count stability",
    },
    ScenarioDefinition {
        scenario_id: "pair-secondary-supplementary",
        focus: "proper-pair/secondary/supplementary counting",
    },
    ScenarioDefinition {
        scenario_id: "mapq-distribution-stability",
        focus: "MAPQ distribution stability",
    },
];

fn mapping_summary_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("mapping_summary_rows_present={}", !rows.is_empty()),
        format!(
            "mapping_summary_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
        ),
        format!(
            "mapping_summary_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class == "ready").count()
        ),
        format!(
            "mapping_summary_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.appraiser_id == "artifact-validity"
                        || finding.appraiser_id == "scientific-output"
                })
                .count()
        ),
    ]
}

const COVERAGE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sparse-lowpass-regimes",
        focus: "sparse and low-pass coverage regimes",
    },
    ScenarioDefinition {
        scenario_id: "target-wgs-regimes",
        focus: "target-like and WGS-like coverage regimes",
    },
    ScenarioDefinition {
        scenario_id: "coverage-refusal-boundary",
        focus: "unusable coverage refusal boundary",
    },
];

fn coverage_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("coverage_rows_present={}", !rows.is_empty()),
        format!("coverage_stage_bound={}", rows.iter().any(|row| row.stage_id == "bam.coverage")),
        format!(
            "coverage_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
        format!(
            "coverage_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.failure_class.contains("readiness")).count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G129",
        title: "benchmark mapping summaries",
        stage_ids: &["bam.mapping_summary"],
        scenarios: MAPPING_SUMMARY_SCENARIOS,
        checks: mapping_summary_checks,
    },
    GoalDefinition {
        goal_id: "G130",
        title: "benchmark coverage regimes",
        stage_ids: &["bam.coverage"],
        scenarios: COVERAGE_SCENARIOS,
        checks: coverage_checks,
    },
];
