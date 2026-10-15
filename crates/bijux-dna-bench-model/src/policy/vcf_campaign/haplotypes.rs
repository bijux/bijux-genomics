//! Owner: pure VCF campaign haplotypes declarations and checks.
use super::{BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition};

const G150_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "phasing-panel-map-prereq",
        focus: "panel and map prerequisite enforcement",
    },
    ScenarioDefinition {
        scenario_id: "phasing-sample-prereq",
        focus: "sample-level phasing prerequisites",
    },
    ScenarioDefinition {
        scenario_id: "phasing-confidence-caveats",
        focus: "phasing confidence and caveat boundaries",
    },
];

fn phasing_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("phasing_rows_present={}", !rows.is_empty()),
        format!(
            "phasing_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!("phasing_stage_bound={}", rows.iter().any(|row| row.stage_id == "vcf.phasing")),
        format!(
            "phasing_panel_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.prepare_reference_panel")
        ),
        format!(
            "phasing_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

const G151_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "panel-map-identity",
        focus: "panel and map identity compatibility",
    },
    ScenarioDefinition {
        scenario_id: "simulation-vs-production-labels",
        focus: "simulation and production label separation",
    },
    ScenarioDefinition {
        scenario_id: "imputation-confidence-surface",
        focus: "imputation confidence output surface",
    },
];

fn imputation_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("imputation_rows_present={}", !rows.is_empty()),
        format!(
            "imputation_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "imputation_impute_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.impute")
        ),
        format!(
            "imputation_panel_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.prepare_reference_panel")
        ),
        format!(
            "imputation_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G150",
        title: "benchmark phasing workflows",
        stage_ids: &["vcf.phasing", "vcf.prepare_reference_panel"],
        scenarios: G150_SCENARIOS,
        checks: phasing_checks,
    },
    GoalDefinition {
        goal_id: "G151",
        title: "benchmark imputation workflows",
        stage_ids: &[
            "vcf.impute",
            "vcf.imputation_metrics",
            "vcf.prepare_reference_panel",
            "vcf.postprocess",
        ],
        scenarios: G151_SCENARIOS,
        checks: imputation_checks,
    },
];
