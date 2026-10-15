//! Owner: pure FASTQ campaign validation declarations and checks.
use super::{BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition};

const G101_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "malformed-fastq",
        focus: "malformed records and parse refusal",
    },
    ScenarioDefinition {
        scenario_id: "compressed-layouts",
        focus: "gzip and mixed compression handling",
    },
    ScenarioDefinition { scenario_id: "truncated-stream", focus: "truncated file detection" },
    ScenarioDefinition {
        scenario_id: "paired-interleaved-mixed",
        focus: "paired and interleaved layout validation",
    },
    ScenarioDefinition {
        scenario_id: "large-fastq",
        focus: "large input scale for validation throughput",
    },
];

fn validation_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, queue_entries, .. } = inputs;
    vec![
        format!("validation_rows_present={}", !rows.is_empty()),
        format!(
            "layout_diversity_detected={}",
            rows.iter()
                .any(|row| row.stage_id == "fastq.validate_reads" && row.tool_id != "<unbound>")
        ),
        format!(
            "validation_failure_classes={}",
            findings
                .iter()
                .map(|finding| finding.failure_class.clone())
                .collect::<BTreeSet<_>>()
                .len()
        ),
        format!(
            "validation_queue_entries={}",
            queue_entries
                .iter()
                .filter(|entry| entry.failure_class.contains("runtime")
                    || entry.failure_class.contains("readiness"))
                .count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[GoalDefinition {
    goal_id: "G101",
    title: "benchmark FASTQ validation",
    stage_ids: &["fastq.validate_reads"],
    scenarios: G101_SCENARIOS,
    checks: validation_checks,
}];
