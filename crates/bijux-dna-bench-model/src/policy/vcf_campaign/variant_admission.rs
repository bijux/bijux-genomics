//! Owner: pure VCF campaign variant admission declarations and checks.
use super::{BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition};

const G141_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "header-format-boundaries",
        focus: "malformed header and INFO/FORMAT boundaries",
    },
    ScenarioDefinition {
        scenario_id: "sample-and-sort-boundaries",
        focus: "duplicate samples and unsorted record handling",
    },
    ScenarioDefinition {
        scenario_id: "index-reference-mismatch",
        focus: "index validity and reference mismatch refusal",
    },
];

fn validation_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("validation_rows_present={}", !rows.is_empty()),
        format!(
            "validation_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "validation_call_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.call")
        ),
        format!(
            "validation_filter_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.filter")
        ),
        format!(
            "validation_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "validation_failure_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "failure-class").count()
        ),
    ]
}

const G142_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "core-count-summaries",
        focus: "variant counts and pass-rate summaries",
    },
    ScenarioDefinition {
        scenario_id: "missingness-het-titv",
        focus: "missingness, heterozygosity, and Ti/Tv stability",
    },
    ScenarioDefinition { scenario_id: "sample-level-stats", focus: "per-sample summary stability" },
];

fn stats_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("stats_rows_present={}", !rows.is_empty()),
        format!("stats_stage_bound={}", rows.iter().any(|row| row.stage_id == "vcf.stats")),
        format!(
            "stats_degraded_rows={}",
            rows.iter().filter(|row| row.readiness_class == "degraded").count()
        ),
        format!(
            "stats_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class == "ready").count()
        ),
        format!(
            "stats_scientific_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "scientific-output").count()
        ),
    ]
}

const G143_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "site-sample-filter-scope",
        focus: "site and sample scoped filter expressions",
    },
    ScenarioDefinition {
        scenario_id: "missingness-threshold-boundary",
        focus: "missingness threshold boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "subset-identity-guard",
        focus: "subset identity and sample list stability",
    },
];

fn filtering_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("filter_rows_present={}", !rows.is_empty()),
        format!("filter_stage_bound={}", rows.iter().any(|row| row.stage_id == "vcf.filter")),
        format!(
            "filter_degraded_rows={}",
            rows.iter().filter(|row| row.readiness_class == "degraded").count()
        ),
        format!(
            "filter_repetition_floor={}",
            rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
        ),
        format!(
            "filter_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.failure_class.contains("readiness")).count()
        ),
    ]
}

const G144_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "left-normalization-boundary",
        focus: "left-normalization boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "multiallelic-decomposition",
        focus: "multiallelic decomposition and duplicate handling",
    },
    ScenarioDefinition {
        scenario_id: "raw-normalized-diff",
        focus: "raw-to-normalized genotype retention diff",
    },
];

fn normalization_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("normalization_rows_present={}", !rows.is_empty()),
        format!(
            "normalization_postprocess_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.postprocess")
        ),
        format!(
            "normalization_repetition_floor={}",
            rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
        ),
        format!(
            "normalization_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
        format!(
            "normalization_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.appraiser_id == "artifact-validity"
                        || finding.appraiser_id == "scientific-output"
                })
                .count()
        ),
    ]
}

const G145_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "build-and-contig-aliases",
        focus: "build identity and contig alias compatibility",
    },
    ScenarioDefinition {
        scenario_id: "fasta-and-index-context",
        focus: "FASTA checksum and index state validation",
    },
    ScenarioDefinition {
        scenario_id: "panel-map-compatibility",
        focus: "panel and genetic map compatibility boundaries",
    },
];

fn reference_context_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("reference_context_rows_present={}", !rows.is_empty()),
        format!(
            "reference_context_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "reference_context_call_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.call")
        ),
        format!(
            "reference_context_panel_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.prepare_reference_panel")
        ),
        format!(
            "reference_context_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.postprocess")
        ),
        format!(
            "reference_context_failure_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "failure-class").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G141",
        title: "benchmark VCF validation",
        stage_ids: &["vcf.call", "vcf.filter"],
        scenarios: G141_SCENARIOS,
        checks: validation_checks,
    },
    GoalDefinition {
        goal_id: "G142",
        title: "benchmark VCF stats",
        stage_ids: &["vcf.stats"],
        scenarios: G142_SCENARIOS,
        checks: stats_checks,
    },
    GoalDefinition {
        goal_id: "G143",
        title: "benchmark VCF filtering",
        stage_ids: &["vcf.filter"],
        scenarios: G143_SCENARIOS,
        checks: filtering_checks,
    },
    GoalDefinition {
        goal_id: "G144",
        title: "benchmark VCF normalization",
        stage_ids: &["vcf.postprocess"],
        scenarios: G144_SCENARIOS,
        checks: normalization_checks,
    },
    GoalDefinition {
        goal_id: "G145",
        title: "benchmark VCF reference context",
        stage_ids: &["vcf.call", "vcf.prepare_reference_panel", "vcf.postprocess"],
        scenarios: G145_SCENARIOS,
        checks: reference_context_checks,
    },
];
