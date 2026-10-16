//! Owner: pure FASTQ campaign depletion declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use bijux_dna_core::id_catalog::{
    FASTQ_DEPLETE_HOST, FASTQ_DEPLETE_REFERENCE_CONTAMINANTS, FASTQ_DEPLETE_RRNA, FASTQ_SCREEN,
};

const G112_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "host-reference-identity",
        focus: "host reference identity and version boundaries",
    },
    ScenarioDefinition {
        scenario_id: "retained-rejected-balance",
        focus: "retained vs rejected read artifact accounting",
    },
    ScenarioDefinition {
        scenario_id: "depletion-failure-signals",
        focus: "host depletion refusal and failure signatures",
    },
];

fn host_depletion_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("host_depletion_rows_present={}", !rows.is_empty()),
        format!(
            "host_depletion_stage_bound={}",
            rows.iter().any(|row| row.stage_id == FASTQ_DEPLETE_HOST)
        ),
        format!(
            "host_depletion_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "host_depletion_db_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class.contains("readiness")
                        || finding.failure_class.contains("corpus")
                })
                .count()
        ),
    ]
}

const G113_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "rrna-db-versions",
        focus: "rRNA database version comparison",
    },
    ScenarioDefinition {
        scenario_id: "rrna-layout-effects",
        focus: "layout-specific retained/rejected read behavior",
    },
    ScenarioDefinition {
        scenario_id: "rrna-caveat-surface",
        focus: "rRNA depletion caveat reporting stability",
    },
];

fn rrna_depletion_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("rrna_depletion_rows_present={}", !rows.is_empty()),
        format!(
            "rrna_depletion_stage_bound={}",
            rows.iter().any(|row| row.stage_id == FASTQ_DEPLETE_RRNA)
        ),
        format!(
            "rrna_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class == "ready").count()
        ),
        format!(
            "rrna_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.failure_class.contains("readiness")).count()
        ),
    ]
}

const G114_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "contaminant-db-versions",
        focus: "contaminant database version benchmarking",
    },
    ScenarioDefinition {
        scenario_id: "contaminant-false-positive-pressure",
        focus: "false positive pressure and retained read impact",
    },
    ScenarioDefinition {
        scenario_id: "contaminant-caveat-propagation",
        focus: "contaminant caveat propagation semantics",
    },
];

fn contaminant_depletion_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, queue_entries, .. } = inputs;
    vec![
        format!("contaminant_depletion_rows_present={}", !rows.is_empty()),
        format!(
            "contaminant_stage_bound={}",
            rows.iter().any(|row| row.stage_id == FASTQ_DEPLETE_REFERENCE_CONTAMINANTS)
        ),
        format!(
            "contaminant_critical_findings={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
        format!(
            "contaminant_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.failure_class.contains("readiness")).count()
        ),
    ]
}

const G115_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "taxonomy-rank-behavior",
        focus: "taxonomy rank assignment behavior",
    },
    ScenarioDefinition {
        scenario_id: "advisory-only-guard",
        focus: "advisory-only classification guardrails",
    },
    ScenarioDefinition {
        scenario_id: "taxonomy-report-stability",
        focus: "taxonomy report stability across runs",
    },
];

fn taxonomy_screening_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, queue_entries, .. } = inputs;
    vec![
        format!("taxonomy_rows_present={}", !rows.is_empty()),
        format!("taxonomy_stage_bound={}", rows.iter().any(|row| row.stage_id == FASTQ_SCREEN)),
        format!(
            "taxonomy_advisory_signals={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.appraiser_id == "scientific-output"
                        || finding.appraiser_id == "failure-class"
                })
                .count()
        ),
        format!(
            "taxonomy_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.failure_class.contains("readiness")).count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G112",
        title: "benchmark host depletion",
        stage_ids: &[FASTQ_DEPLETE_HOST],
        scenarios: G112_SCENARIOS,
        checks: host_depletion_checks,
    },
    GoalDefinition {
        goal_id: "G113",
        title: "benchmark rRNA depletion",
        stage_ids: &[FASTQ_DEPLETE_RRNA],
        scenarios: G113_SCENARIOS,
        checks: rrna_depletion_checks,
    },
    GoalDefinition {
        goal_id: "G114",
        title: "benchmark contaminant depletion",
        stage_ids: &[FASTQ_DEPLETE_REFERENCE_CONTAMINANTS],
        scenarios: G114_SCENARIOS,
        checks: contaminant_depletion_checks,
    },
    GoalDefinition {
        goal_id: "G115",
        title: "benchmark taxonomy screening",
        stage_ids: &[FASTQ_SCREEN],
        scenarios: G115_SCENARIOS,
        checks: taxonomy_screening_checks,
    },
];
