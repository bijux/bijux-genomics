//! Owner: pure FASTQ campaign sampling declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};

const G107_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition { scenario_id: "count-mode", focus: "count-based subsampling correctness" },
    ScenarioDefinition {
        scenario_id: "fraction-mode",
        focus: "fraction-based subsampling consistency",
    },
    ScenarioDefinition {
        scenario_id: "seeded-pair-preservation",
        focus: "seed reproducibility with pair preservation",
    },
];

fn subsample_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("subsample_rows_present={}", !rows.is_empty()),
        format!(
            "subsample_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.subsample_reads")
        ),
        format!(
            "subsample_repetition_floor={}",
            rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
        ),
        format!(
            "subsample_repro_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "reproducibility").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[GoalDefinition {
    goal_id: "G107",
    title: "benchmark FASTQ subsampling",
    stage_ids: &["fastq.subsample_reads"],
    scenarios: G107_SCENARIOS,
    checks: subsample_checks,
}];
