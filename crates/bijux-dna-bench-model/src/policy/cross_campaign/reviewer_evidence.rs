//! Owner: pure cross-domain campaign reviewer evidence declarations and checks.
use super::{
    BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition, BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
    BRIDGE_FASTQ_TRIM_TO_VCF_GL, BRIDGE_VCF_FILTER_TO_VCF_STATS,
};

const G170_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "reviewer-bundle-export",
        focus: "encrypted result/code bundle export integrity",
    },
    ScenarioDefinition {
        scenario_id: "reviewer-bundle-decrypt-import",
        focus: "local decrypt/import validation boundaries",
    },
    ScenarioDefinition {
        scenario_id: "reviewer-bundle-cross-domain-audit",
        focus: "cross-domain reviewer audit continuity",
    },
];

fn reviewer_bundle_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries } = inputs;
    vec![
        format!("reviewer_bundle_rows_present={}", !rows.is_empty()),
        format!(
            "reviewer_bundle_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "reviewer_bundle_gl_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_TRIM_TO_VCF_GL)
        ),
        format!(
            "reviewer_bundle_call_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_BAM_GENOTYPING_TO_VCF_CALL)
        ),
        format!(
            "reviewer_bundle_hardening_entries={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[GoalDefinition {
    goal_id: "G170",
    title: "benchmark cross-domain reviewer bundle generation",
    stage_ids: &[
        BRIDGE_FASTQ_TRIM_TO_VCF_GL,
        BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
        BRIDGE_VCF_FILTER_TO_VCF_STATS,
    ],
    scenarios: G170_SCENARIOS,
    checks: reviewer_bundle_checks,
}];
