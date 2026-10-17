//! Owner: pure cross-domain campaign source integrity declarations and checks.
use super::{
    BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition,
    BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER, BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
    BRIDGE_BAM_SUMMARY_TO_VCF_STATS, BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN,
    BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION, BRIDGE_VCF_FILTER_TO_VCF_STATS,
};

const G167_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sample-sheet-good-vs-bad",
        focus: "good and bad sample sheet validation boundaries",
    },
    ScenarioDefinition {
        scenario_id: "sample-sheet-duplicate-missing",
        focus: "duplicate samples and missing file refusal",
    },
    ScenarioDefinition {
        scenario_id: "sample-sheet-reference-mode-conflicts",
        focus: "reference and mode conflict refusal behavior",
    },
];

fn sample_sheet_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("sample_sheet_rows_present={}", !rows.is_empty()),
        format!(
            "sample_sheet_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "sample_sheet_validate_align_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN)
        ),
        format!(
            "sample_sheet_validate_contamination_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION)
        ),
        format!(
            "sample_sheet_degraded_rows={}",
            rows.iter().filter(|row| row.readiness_class == "degraded").count()
        ),
    ]
}

const G168_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "reference-index-change-before-after",
        focus: "before/after reference and index change propagation",
    },
    ScenarioDefinition {
        scenario_id: "panel-map-alias-invalidation",
        focus: "panel/map/alias invalidation behavior",
    },
    ScenarioDefinition {
        scenario_id: "cross-domain-invalidation-report",
        focus: "cross-domain invalidation report continuity",
    },
];

fn reference_change_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("reference_change_rows_present={}", !rows.is_empty()),
        format!(
            "reference_change_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "reference_change_call_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_BAM_GENOTYPING_TO_VCF_CALL)
        ),
        format!(
            "reference_change_stats_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_BAM_SUMMARY_TO_VCF_STATS)
        ),
        format!(
            "reference_change_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
    ]
}

const G169_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "contamination-fastq-to-bam",
        focus: "contamination propagation from FASTQ to BAM",
    },
    ScenarioDefinition {
        scenario_id: "contamination-bam-to-vcf",
        focus: "contamination propagation from BAM to VCF",
    },
    ScenarioDefinition {
        scenario_id: "contamination-population-caveat-flow",
        focus: "population caveat propagation from contamination signals",
    },
];

fn contamination_flow_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("contamination_flow_rows_present={}", !rows.is_empty()),
        format!(
            "contamination_flow_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "contamination_flow_fastq_to_bam_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION)
        ),
        format!(
            "contamination_flow_bam_to_vcf_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER)
        ),
        format!(
            "contamination_flow_stats_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_VCF_FILTER_TO_VCF_STATS)
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G167",
        title: "benchmark sample-sheet validation",
        stage_ids: &[
            BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN,
            BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION,
        ],
        scenarios: G167_SCENARIOS,
        checks: sample_sheet_checks,
    },
    GoalDefinition {
        goal_id: "G168",
        title: "benchmark reference-change propagation",
        stage_ids: &[
            BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
            BRIDGE_BAM_SUMMARY_TO_VCF_STATS,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G168_SCENARIOS,
        checks: reference_change_checks,
    },
    GoalDefinition {
        goal_id: "G169",
        title: "benchmark contamination propagation",
        stage_ids: &[
            BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION,
            BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G169_SCENARIOS,
        checks: contamination_flow_checks,
    },
];
