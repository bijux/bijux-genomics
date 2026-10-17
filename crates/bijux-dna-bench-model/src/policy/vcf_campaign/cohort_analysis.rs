//! Owner: pure VCF campaign cohort analysis declarations and checks.
use super::{BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition};

const G152_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sample-qc-and-missingness",
        focus: "sample QC and missingness boundaries",
    },
    ScenarioDefinition {
        scenario_id: "heterozygosity-relatedness-flags",
        focus: "heterozygosity and relatedness flag behavior",
    },
    ScenarioDefinition {
        scenario_id: "cohort-filter-impact",
        focus: "sample-count and filter impact on cohort QC",
    },
];

fn cohort_qc_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("cohort_qc_rows_present={}", !rows.is_empty()),
        format!(
            "cohort_qc_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!("cohort_qc_qc_stage_bound={}", rows.iter().any(|row| row.stage_id == "vcf.qc")),
        format!(
            "cohort_qc_stats_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.stats")
        ),
        format!(
            "cohort_qc_filter_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.filter")
        ),
    ]
}

const G153_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "ld-pruning-boundary",
        focus: "LD pruning and marker count boundary",
    },
    ScenarioDefinition {
        scenario_id: "small-cohort-caveats",
        focus: "small cohort caveat and refusal behavior",
    },
    ScenarioDefinition {
        scenario_id: "sample-inclusion-guardrail",
        focus: "sample inclusion guardrail behavior",
    },
];

fn population_guardrail_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("population_guardrail_rows_present={}", !rows.is_empty()),
        format!(
            "population_guardrail_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "population_guardrail_pca_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.pca")
        ),
        format!(
            "population_guardrail_admixture_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.admixture")
        ),
        format!(
            "population_guardrail_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
    ]
}

const G154_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition { scenario_id: "roh-marker-density", focus: "ROH marker density boundary" },
    ScenarioDefinition {
        scenario_id: "ibd-missingness-cohort-size",
        focus: "IBD missingness and cohort-size boundaries",
    },
    ScenarioDefinition {
        scenario_id: "method-assumption-caveats",
        focus: "ROH/IBD method assumption caveats",
    },
];

fn roh_ibd_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("roh_ibd_rows_present={}", !rows.is_empty()),
        format!(
            "roh_ibd_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!("roh_ibd_roh_stage_bound={}", rows.iter().any(|row| row.stage_id == "vcf.roh")),
        format!("roh_ibd_ibd_stage_bound={}", rows.iter().any(|row| row.stage_id == "vcf.ibd")),
        format!(
            "roh_ibd_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
    ]
}

const G155_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "underpowered-cohort-refusal",
        focus: "underpowered cohort refusal behavior",
    },
    ScenarioDefinition {
        scenario_id: "missing-assumption-refusal",
        focus: "missing demography assumption refusal",
    },
    ScenarioDefinition {
        scenario_id: "incompatible-method-caveats",
        focus: "incompatible demography method caveats",
    },
];

fn demography_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("demography_rows_present={}", !rows.is_empty()),
        format!(
            "demography_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "demography_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.demography")
        ),
        format!(
            "demography_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "demography_critical_findings={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
    ]
}

const G158_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "population-handoff-trust-classes",
        focus: "population handoff trust-class propagation",
    },
    ScenarioDefinition {
        scenario_id: "filtered-normalized-handoff",
        focus: "filtered and normalized handoff boundaries",
    },
    ScenarioDefinition {
        scenario_id: "downstream-stage-caveat-flow",
        focus: "downstream caveat flow across population stages",
    },
];

fn population_handoff_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("population_handoff_rows_present={}", !rows.is_empty()),
        format!(
            "population_handoff_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "population_handoff_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.postprocess")
        ),
        format!(
            "population_handoff_demography_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.demography")
        ),
        format!(
            "population_handoff_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G152",
        title: "benchmark cohort QC",
        stage_ids: &["vcf.qc", "vcf.stats", "vcf.filter"],
        scenarios: G152_SCENARIOS,
        checks: cohort_qc_checks,
    },
    GoalDefinition {
        goal_id: "G153",
        title: "benchmark PCA/admixture guardrails",
        stage_ids: &["vcf.pca", "vcf.admixture", "vcf.population_structure", "vcf.filter"],
        scenarios: G153_SCENARIOS,
        checks: population_guardrail_checks,
    },
    GoalDefinition {
        goal_id: "G154",
        title: "benchmark ROH/IBD boundaries",
        stage_ids: &["vcf.roh", "vcf.ibd", "vcf.filter"],
        scenarios: G154_SCENARIOS,
        checks: roh_ibd_checks,
    },
    GoalDefinition {
        goal_id: "G155",
        title: "benchmark demography boundaries",
        stage_ids: &["vcf.demography", "vcf.ibd", "vcf.roh"],
        scenarios: G155_SCENARIOS,
        checks: demography_checks,
    },
    GoalDefinition {
        goal_id: "G158",
        title: "benchmark population-analysis handoff",
        stage_ids: &["vcf.postprocess", "vcf.pca", "vcf.roh", "vcf.ibd", "vcf.demography"],
        scenarios: G158_SCENARIOS,
        checks: population_handoff_checks,
    },
];
