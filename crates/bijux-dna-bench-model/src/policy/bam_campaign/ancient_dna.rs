//! Owner: pure BAM campaign ancient dna goal declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};

const DAMAGE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "damage-fragmentation-signals",
        focus: "terminal damage and fragmentation evidence",
    },
    ScenarioDefinition {
        scenario_id: "low-endogenous-caveats",
        focus: "low endogenous content caveat behavior",
    },
    ScenarioDefinition {
        scenario_id: "adna-damage-refusal-boundary",
        focus: "aDNA damage refusal and caveat boundary",
    },
];

fn damage_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("adna_damage_rows_present={}", !rows.is_empty()),
        format!("adna_damage_stage_bound={}", rows.iter().any(|row| row.stage_id == "bam.damage")),
        format!(
            "adna_damage_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class == "ready").count()
        ),
        format!(
            "adna_damage_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "scientific-output").count()
        ),
    ]
}

const AUTHENTICITY_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "pmd-evidence-surface",
        focus: "PMD-like evidence reporting surface",
    },
    ScenarioDefinition {
        scenario_id: "authenticity-overclaim-guard",
        focus: "authenticity over-claim guardrails",
    },
    ScenarioDefinition {
        scenario_id: "authenticity-caveat-stability",
        focus: "authenticity caveat stability across repeats",
    },
];

fn authenticity_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("authenticity_rows_present={}", !rows.is_empty()),
        format!(
            "authenticity_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.authenticity")
        ),
        format!(
            "authenticity_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
        format!(
            "authenticity_caveat_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "failure-class").count()
        ),
    ]
}

const ENDOGENOUS_CONTENT_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "prealign-endogenous-interpretation",
        focus: "prealignment endogenous-content interpretation",
    },
    ScenarioDefinition {
        scenario_id: "postalign-endogenous-interpretation",
        focus: "postalignment endogenous-content interpretation",
    },
    ScenarioDefinition {
        scenario_id: "contaminant-interaction-effects",
        focus: "contaminant interaction effects on endogenous estimates",
    },
];

fn endogenous_content_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("endogenous_rows_present={}", !rows.is_empty()),
        format!(
            "endogenous_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.endogenous_content")
        ),
        format!(
            "endogenous_degraded_rows={}",
            rows.iter().filter(|row| row.readiness_class == "degraded").count()
        ),
        format!(
            "endogenous_scientific_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "scientific-output").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G131",
        title: "benchmark ancient-DNA damage evidence",
        stage_ids: &["bam.damage"],
        scenarios: DAMAGE_SCENARIOS,
        checks: damage_checks,
    },
    GoalDefinition {
        goal_id: "G132",
        title: "benchmark PMD/authenticity advisory outputs",
        stage_ids: &["bam.authenticity"],
        scenarios: AUTHENTICITY_SCENARIOS,
        checks: authenticity_checks,
    },
    GoalDefinition {
        goal_id: "G135",
        title: "benchmark endogenous-content estimation",
        stage_ids: &["bam.endogenous_content"],
        scenarios: ENDOGENOUS_CONTENT_SCENARIOS,
        checks: endogenous_content_checks,
    },
];
