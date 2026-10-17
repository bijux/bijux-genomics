//! Owner: pure cross-domain campaign execution policy declarations and checks.
use super::{
    BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition, BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
    BRIDGE_BAM_SUMMARY_TO_VCF_STATS, BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY,
    BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN, BRIDGE_FASTQ_TRIM_TO_VCF_GL, BRIDGE_VCF_FILTER_TO_VCF_STATS,
};

const G165_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "batch-fanout-per-sample",
        focus: "per-sample fan-out boundaries",
    },
    ScenarioDefinition {
        scenario_id: "batch-fanin-cohort-aggregate",
        focus: "cohort fan-in and aggregate report boundaries",
    },
    ScenarioDefinition {
        scenario_id: "batch-lineage-overwrite-guard",
        focus: "lineage continuity and overwrite guardrails",
    },
];

fn batch_fan_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("batch_fan_rows_present={}", !rows.is_empty()),
        format!(
            "batch_fan_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "batch_fan_fastq_align_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN)
        ),
        format!(
            "batch_fan_summary_stats_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_BAM_SUMMARY_TO_VCF_STATS)
        ),
        format!(
            "batch_fan_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
    ]
}

const G166_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "partial-failure-fastq-boundary",
        focus: "partial failure behavior when FASTQ stage fails",
    },
    ScenarioDefinition {
        scenario_id: "partial-failure-bam-boundary",
        focus: "partial failure behavior when BAM stage fails",
    },
    ScenarioDefinition {
        scenario_id: "partial-failure-vcf-boundary",
        focus: "partial failure behavior when VCF stage fails",
    },
];

fn partial_failure_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("partial_failure_rows_present={}", !rows.is_empty()),
        format!(
            "partial_failure_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "partial_failure_fastq_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN)
        ),
        format!(
            "partial_failure_bam_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_BAM_GENOTYPING_TO_VCF_CALL)
        ),
        format!(
            "partial_failure_vcf_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_TRIM_TO_VCF_GL)
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G165",
        title: "benchmark batch fan-out/fan-in",
        stage_ids: &[
            BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN,
            BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY,
            BRIDGE_BAM_SUMMARY_TO_VCF_STATS,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G165_SCENARIOS,
        checks: batch_fan_checks,
    },
    GoalDefinition {
        goal_id: "G166",
        title: "benchmark partial failure policy",
        stage_ids: &[
            BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN,
            BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
            BRIDGE_FASTQ_TRIM_TO_VCF_GL,
        ],
        scenarios: G166_SCENARIOS,
        checks: partial_failure_checks,
    },
];
