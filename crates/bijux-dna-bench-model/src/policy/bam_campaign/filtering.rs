//! Owner: pure BAM campaign filtering goal declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};

const DUPLICATES_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "mark-vs-remove-modes",
        focus: "mark-only vs remove duplicate modes",
    },
    ScenarioDefinition {
        scenario_id: "picard-vs-samtools-like",
        focus: "method differences across duplicate handlers",
    },
    ScenarioDefinition {
        scenario_id: "umi-aware-duplicate-policy",
        focus: "UMI-aware duplicate handling boundaries",
    },
];

fn duplicates_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("duplicate_rows_present={}", !rows.is_empty()),
        format!(
            "duplicate_markdup_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.markdup")
        ),
        format!(
            "duplicate_metrics_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.duplication_metrics")
        ),
        format!(
            "duplicate_method_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "scientific-output").count()
        ),
    ]
}

const MAPPING_QUALITY_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "threshold-impact-surface",
        focus: "MAPQ threshold impact surface",
    },
    ScenarioDefinition {
        scenario_id: "retained-removed-coverage-effect",
        focus: "retained/removed read coverage effect",
    },
    ScenarioDefinition {
        scenario_id: "vcf-readiness-impact",
        focus: "downstream VCF readiness impact",
    },
];

fn mapping_quality_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("mapq_rows_present={}", !rows.is_empty()),
        format!("mapq_stage_bound={}", rows.iter().any(|row| row.stage_id == "bam.mapq_filter")),
        format!(
            "mapq_repetition_floor={}",
            rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
        ),
        format!(
            "mapq_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "scientific-output").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G127",
        title: "benchmark duplicate handling",
        stage_ids: &["bam.markdup", "bam.duplication_metrics"],
        scenarios: DUPLICATES_SCENARIOS,
        checks: duplicates_checks,
    },
    GoalDefinition {
        goal_id: "G128",
        title: "benchmark MAPQ filtering",
        stage_ids: &["bam.mapq_filter"],
        scenarios: MAPPING_QUALITY_SCENARIOS,
        checks: mapping_quality_checks,
    },
];
