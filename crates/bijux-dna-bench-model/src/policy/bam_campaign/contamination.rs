//! Owner: pure BAM campaign contamination goal declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};

const MITOCHONDRIAL_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "mito-contam-prerequisites",
        focus: "mitochondrial contamination prerequisites",
    },
    ScenarioDefinition {
        scenario_id: "mito-contam-confidence",
        focus: "confidence and caveat behavior for mito contamination",
    },
    ScenarioDefinition {
        scenario_id: "mito-contam-failure-modes",
        focus: "mitochondrial contamination failure modes",
    },
];

fn mitochondrial_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("mito_contam_rows_present={}", !rows.is_empty()),
        format!(
            "mito_contam_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.contamination")
        ),
        format!(
            "mito_contam_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class == "ready").count()
        ),
        format!(
            "mito_contam_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.appraiser_id == "scientific-output"
                        || finding.appraiser_id == "failure-class"
                })
                .count()
        ),
    ]
}

const NUCLEAR_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "nuclear-panel-build-compat",
        focus: "nuclear contamination panel/build compatibility",
    },
    ScenarioDefinition {
        scenario_id: "nuclear-coverage-constraints",
        focus: "coverage constraints for nuclear contamination",
    },
    ScenarioDefinition {
        scenario_id: "nuclear-estimator-output-boundary",
        focus: "estimator output and caveat boundary",
    },
];

fn nuclear_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("nuclear_contam_rows_present={}", !rows.is_empty()),
        format!(
            "nuclear_contam_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.contamination")
        ),
        format!(
            "nuclear_contam_degraded_rows={}",
            rows.iter().filter(|row| row.readiness_class == "degraded").count()
        ),
        format!(
            "nuclear_contam_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.failure_class.contains("readiness")).count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G133",
        title: "benchmark mitochondrial contamination",
        stage_ids: &["bam.contamination"],
        scenarios: MITOCHONDRIAL_SCENARIOS,
        checks: mitochondrial_checks,
    },
    GoalDefinition {
        goal_id: "G134",
        title: "benchmark nuclear contamination",
        stage_ids: &["bam.contamination"],
        scenarios: NUCLEAR_SCENARIOS,
        checks: nuclear_checks,
    },
];
