//! Owner: pure VCF campaign evidence declarations and checks.
use super::{BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition};

const G159_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "stats-filter-scale",
        focus: "stats and filter scale behavior",
    },
    ScenarioDefinition {
        scenario_id: "normalize-annotation-scale",
        focus: "normalization and annotation scale behavior",
    },
    ScenarioDefinition {
        scenario_id: "runtime-memory-evidence-size",
        focus: "runtime/memory/evidence-size pressure",
    },
];

fn large_file_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("large_file_rows_present={}", !rows.is_empty()),
        format!(
            "large_file_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!("large_file_impute_bound={}", rows.iter().any(|row| row.stage_id == "vcf.impute")),
        format!(
            "large_file_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
        format!(
            "large_file_runtime_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "runtime-performance").count()
        ),
    ]
}

const G160_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "validation-normalization-template",
        focus: "validation and normalization template baseline",
    },
    ScenarioDefinition {
        scenario_id: "cohort-imputation-template",
        focus: "cohort QC and imputation template baseline",
    },
    ScenarioDefinition {
        scenario_id: "population-demography-template",
        focus: "population, ROH/IBD, and demography template baseline",
    },
];

fn template_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("full_template_rows_present={}", !rows.is_empty()),
        format!(
            "full_template_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "full_template_core_stages_covered={}",
            rows.iter()
                .filter(|row| {
                    matches!(
                        row.stage_id.as_str(),
                        "vcf.call"
                            | "vcf.filter"
                            | "vcf.stats"
                            | "vcf.qc"
                            | "vcf.impute"
                            | "vcf.pca"
                            | "vcf.roh"
                            | "vcf.ibd"
                            | "vcf.demography"
                            | "vcf.phasing"
                    )
                })
                .count()
        ),
        format!(
            "full_template_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
        format!(
            "full_template_hardening_entries={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G159",
        title: "benchmark VCF large-file behavior",
        stage_ids: &["vcf.stats", "vcf.filter", "vcf.postprocess", "vcf.impute"],
        scenarios: G159_SCENARIOS,
        checks: large_file_checks,
    },
    GoalDefinition {
        goal_id: "G160",
        title: "benchmark full VCF templates",
        stage_ids: &[
            "vcf.call",
            "vcf.filter",
            "vcf.stats",
            "vcf.qc",
            "vcf.impute",
            "vcf.pca",
            "vcf.roh",
            "vcf.ibd",
            "vcf.demography",
            "vcf.phasing",
        ],
        scenarios: G160_SCENARIOS,
        checks: template_checks,
    },
];
