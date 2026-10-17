//! Owner: pure VCF campaign calling declarations and checks.
use super::{BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition};

const G146_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "damage-remove-policy",
        focus: "damage remove policy boundary",
    },
    ScenarioDefinition { scenario_id: "damage-mask-policy", focus: "damage mask policy boundary" },
    ScenarioDefinition {
        scenario_id: "damage-annotate-policy",
        focus: "damage annotate policy and caveats",
    },
];

fn damage_filtering_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("damage_filter_rows_present={}", !rows.is_empty()),
        format!(
            "damage_filter_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.damage_filter")
        ),
        format!(
            "damage_filter_repetition_floor={}",
            rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
        ),
        format!(
            "damage_filter_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class == "ready").count()
        ),
        format!(
            "damage_filter_caveat_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "scientific-output").count()
        ),
    ]
}

const G147_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "diploid-prerequisites",
        focus: "diploid calling prerequisites and refusal boundaries",
    },
    ScenarioDefinition {
        scenario_id: "diploid-ploidy-reference",
        focus: "ploidy and reference compatibility",
    },
    ScenarioDefinition {
        scenario_id: "diploid-confidence-boundary",
        focus: "diploid confidence and caveat boundary",
    },
];

fn diploid_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, queue_entries, .. } = inputs;
    vec![
        format!("diploid_rows_present={}", !rows.is_empty()),
        format!(
            "diploid_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.call_diploid")
        ),
        format!(
            "diploid_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.failure_class.contains("readiness")).count()
        ),
        format!(
            "diploid_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
        format!(
            "diploid_refusal_findings={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
    ]
}

const G148_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "pseudo-low-coverage-assumptions",
        focus: "low-coverage pseudohaploid assumptions",
    },
    ScenarioDefinition {
        scenario_id: "pseudo-adna-assumptions",
        focus: "aDNA pseudohaploid assumptions and caveats",
    },
    ScenarioDefinition {
        scenario_id: "pseudo-uncertainty-propagation",
        focus: "uncertainty propagation across outputs",
    },
];

fn pseudohaploid_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("pseudohaploid_rows_present={}", !rows.is_empty()),
        format!(
            "pseudohaploid_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.call_pseudohaploid")
        ),
        format!(
            "pseudohaploid_repetition_floor={}",
            rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
        ),
        format!(
            "pseudohaploid_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class == "ready").count()
        ),
        format!(
            "pseudohaploid_caveat_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "failure-class").count()
        ),
    ]
}

const G149_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "gl-caller-outputs",
        focus: "genotype-likelihood caller output integrity",
    },
    ScenarioDefinition {
        scenario_id: "gl-propagation",
        focus: "GL/PL propagation across downstream stages",
    },
    ScenarioDefinition {
        scenario_id: "gl-compatibility",
        focus: "GL-bearing output downstream compatibility",
    },
];

fn genotype_likelihood_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("gl_rows_present={}", !rows.is_empty()),
        format!(
            "gl_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!("gl_call_stage_bound={}", rows.iter().any(|row| row.stage_id == "vcf.call_gl")),
        format!(
            "gl_propagation_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.gl_propagation")
        ),
        format!(
            "gl_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "scientific-output").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G146",
        title: "benchmark damage-aware VCF filtering",
        stage_ids: &["vcf.damage_filter"],
        scenarios: G146_SCENARIOS,
        checks: damage_filtering_checks,
    },
    GoalDefinition {
        goal_id: "G147",
        title: "benchmark diploid-calling boundary",
        stage_ids: &["vcf.call_diploid"],
        scenarios: G147_SCENARIOS,
        checks: diploid_checks,
    },
    GoalDefinition {
        goal_id: "G148",
        title: "benchmark pseudohaploid boundary",
        stage_ids: &["vcf.call_pseudohaploid"],
        scenarios: G148_SCENARIOS,
        checks: pseudohaploid_checks,
    },
    GoalDefinition {
        goal_id: "G149",
        title: "benchmark genotype-likelihood workflows",
        stage_ids: &["vcf.call_gl", "vcf.gl_propagation"],
        scenarios: G149_SCENARIOS,
        checks: genotype_likelihood_checks,
    },
];
