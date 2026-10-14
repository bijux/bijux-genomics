//! Owner: pure BAM campaign identity goal declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};

const SEX_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sex-inference-boundary",
        focus: "sex inference boundary conditions",
    },
    ScenarioDefinition {
        scenario_id: "par-caveats",
        focus: "PAR caveat behavior and missing coverage handling",
    },
    ScenarioDefinition {
        scenario_id: "unsuitable-reference-refusal",
        focus: "unsuitable reference refusal behavior",
    },
];

fn sex_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("sex_par_rows_present={}", !rows.is_empty()),
        format!("sex_par_stage_bound={}", rows.iter().any(|row| row.stage_id == "bam.sex")),
        format!(
            "sex_par_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class == "ready").count()
        ),
        format!(
            "sex_par_caveat_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "failure-class").count()
        ),
    ]
}

const HAPLOGROUPS_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "organelle-y-readiness",
        focus: "organellar and Y evidence readiness",
    },
    ScenarioDefinition {
        scenario_id: "haplogroup-overclaim-guard",
        focus: "haplogroup over-claim prevention",
    },
    ScenarioDefinition {
        scenario_id: "haplogroup-readiness-reporting",
        focus: "haplogroup readiness reporting stability",
    },
];

fn haplogroups_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("haplogroup_rows_present={}", !rows.is_empty()),
        format!(
            "haplogroup_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.haplogroups")
        ),
        format!(
            "haplogroup_degraded_rows={}",
            rows.iter().filter(|row| row.readiness_class == "degraded").count()
        ),
        format!(
            "haplogroup_readiness_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "scientific-output").count()
        ),
    ]
}

const KINSHIP_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "kinship-marker-overlap",
        focus: "kinship marker overlap prerequisites",
    },
    ScenarioDefinition {
        scenario_id: "kinship-coverage-identity",
        focus: "coverage and sample identity prerequisites",
    },
    ScenarioDefinition {
        scenario_id: "kinship-contam-prechecks",
        focus: "contamination precheck behavior for kinship",
    },
];

fn kinship_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("kinship_rows_present={}", !rows.is_empty()),
        format!("kinship_stage_bound={}", rows.iter().any(|row| row.stage_id == "bam.kinship")),
        format!(
            "kinship_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class == "ready").count()
        ),
        format!(
            "kinship_refusal_findings={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G136",
        title: "benchmark sex/PAR evidence",
        stage_ids: &["bam.sex"],
        scenarios: SEX_SCENARIOS,
        checks: sex_checks,
    },
    GoalDefinition {
        goal_id: "G137",
        title: "benchmark haplogroup-readiness",
        stage_ids: &["bam.haplogroups"],
        scenarios: HAPLOGROUPS_SCENARIOS,
        checks: haplogroups_checks,
    },
    GoalDefinition {
        goal_id: "G138",
        title: "benchmark kinship prerequisites",
        stage_ids: &["bam.kinship"],
        scenarios: KINSHIP_SCENARIOS,
        checks: kinship_checks,
    },
];
