//! Owner: pure BAM campaign resources goal declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};

const RESOURCE_ESTIMATION_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "predicted-vs-actual-cpu-mem",
        focus: "predicted vs actual CPU and memory",
    },
    ScenarioDefinition {
        scenario_id: "predicted-vs-actual-io-scratch",
        focus: "predicted vs actual IO and scratch usage",
    },
    ScenarioDefinition {
        scenario_id: "resource-tuning-patch-surface",
        focus: "resource tuning patch surface and impact",
    },
];

fn resource_estimation_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("resource_rows_present={}", !rows.is_empty()),
        format!("resource_qc_pre_bound={}", rows.iter().any(|row| row.stage_id == "bam.qc_pre")),
        format!(
            "resource_summary_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
        ),
        format!(
            "resource_coverage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.coverage")
        ),
        format!(
            "resource_runtime_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "runtime-performance").count()
        ),
    ]
}

const TEMPLATES_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "modern-wgs-template",
        focus: "modern WGS QC template behavior",
    },
    ScenarioDefinition {
        scenario_id: "adna-lowpass-template",
        focus: "aDNA and low-pass readiness templates",
    },
    ScenarioDefinition {
        scenario_id: "targeted-batch-coverage-template",
        focus: "targeted, batch merge, and coverage review templates",
    },
];

fn templates_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("full_template_rows_present={}", !rows.is_empty()),
        format!(
            "full_template_core_stages_covered={}",
            rows.iter()
                .filter(|row| {
                    matches!(
                        row.stage_id.as_str(),
                        "bam.qc_pre"
                            | "bam.align"
                            | "bam.markdup"
                            | "bam.mapping_summary"
                            | "bam.coverage"
                            | "bam.damage"
                            | "bam.authenticity"
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
        goal_id: "G139",
        title: "benchmark BAM resource estimation",
        stage_ids: &["bam.qc_pre", "bam.mapping_summary", "bam.coverage"],
        scenarios: RESOURCE_ESTIMATION_SCENARIOS,
        checks: resource_estimation_checks,
    },
    GoalDefinition {
        goal_id: "G140",
        title: "benchmark full BAM templates",
        stage_ids: &[
            "bam.qc_pre",
            "bam.align",
            "bam.markdup",
            "bam.mapping_summary",
            "bam.coverage",
            "bam.damage",
            "bam.authenticity",
        ],
        scenarios: TEMPLATES_SCENARIOS,
        checks: templates_checks,
    },
];
