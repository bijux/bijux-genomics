//! Owner: pure cross-domain campaign domain handoffs declarations and checks.
use super::{
    BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition,
    BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER, BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
    BRIDGE_BAM_SUMMARY_TO_VCF_STATS, BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE,
    BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY, BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN,
    BRIDGE_FASTQ_TRIM_TO_VCF_GL, BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN,
    BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION, BRIDGE_VCF_FILTER_TO_VCF_STATS,
};

const G161_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "modern-fastq-bam-validate-trim-align",
        focus: "validate, trim, align, summary, and coverage handoff",
    },
    ScenarioDefinition {
        scenario_id: "modern-reference-preflight",
        focus: "modern reference preparation and handoff integrity",
    },
    ScenarioDefinition {
        scenario_id: "modern-handoff-evidence",
        focus: "handoff evidence continuity from FASTQ to BAM",
    },
];

fn modern_workflow_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("modern_workflow_rows_present={}", !rows.is_empty()),
        format!(
            "modern_workflow_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "modern_workflow_fastq_to_bam_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN)
        ),
        format!(
            "modern_workflow_summary_to_stats_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_BAM_SUMMARY_TO_VCF_STATS)
        ),
    ]
}

const G162_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "adna-preprocess-align-damage",
        focus: "aDNA preprocessing, alignment, and damage evidence",
    },
    ScenarioDefinition {
        scenario_id: "adna-contamination-authenticity",
        focus: "aDNA contamination and authenticity caveat flow",
    },
    ScenarioDefinition {
        scenario_id: "adna-cross-domain-appraisal",
        focus: "aDNA cross-domain appraisal consistency",
    },
];

fn ancient_workflow_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("adna_workflow_rows_present={}", !rows.is_empty()),
        format!(
            "adna_workflow_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "adna_workflow_damage_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE)
        ),
        format!(
            "adna_workflow_contamination_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION)
        ),
        format!(
            "adna_workflow_degraded_rows={}",
            rows.iter().filter(|row| row.readiness_class == "degraded").count()
        ),
    ]
}

const G163_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "bam-to-vcf-reference-preflight",
        focus: "reference preflight and callable assumptions",
    },
    ScenarioDefinition {
        scenario_id: "bam-to-vcf-call-filter-stats",
        focus: "call/filter/stats handoff for BAM-to-VCF",
    },
    ScenarioDefinition {
        scenario_id: "bam-to-vcf-handoff-appraisal",
        focus: "BAM-to-VCF handoff appraisal stability",
    },
];

fn bam_to_vcf_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("bam_to_vcf_rows_present={}", !rows.is_empty()),
        format!(
            "bam_to_vcf_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "bam_to_vcf_call_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_BAM_GENOTYPING_TO_VCF_CALL)
        ),
        format!(
            "bam_to_vcf_filter_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER)
        ),
        format!(
            "bam_to_vcf_stats_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_VCF_FILTER_TO_VCF_STATS)
        ),
    ]
}

const G164_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "fastq-to-vcf-raw-to-call",
        focus: "raw FASTQ to VCF calling bridge",
    },
    ScenarioDefinition {
        scenario_id: "fastq-bam-vcf-artifact-preserve",
        focus: "artifact preservation across FASTQ/BAM/VCF surfaces",
    },
    ScenarioDefinition {
        scenario_id: "fastq-to-vcf-full-chain-appraisal",
        focus: "full-chain appraisal consistency",
    },
];

fn fastq_to_vcf_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("fastq_to_vcf_rows_present={}", !rows.is_empty()),
        format!(
            "fastq_to_vcf_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "fastq_to_vcf_gl_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_TRIM_TO_VCF_GL)
        ),
        format!(
            "fastq_to_vcf_call_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_BAM_GENOTYPING_TO_VCF_CALL)
        ),
        format!(
            "fastq_to_vcf_stats_bound={}",
            rows.iter().any(|row| row.stage_id == BRIDGE_VCF_FILTER_TO_VCF_STATS)
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G161",
        title: "benchmark FASTQ-to-BAM modern workflow",
        stage_ids: &[
            BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN,
            BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN,
            BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY,
            BRIDGE_BAM_SUMMARY_TO_VCF_STATS,
        ],
        scenarios: G161_SCENARIOS,
        checks: modern_workflow_checks,
    },
    GoalDefinition {
        goal_id: "G162",
        title: "benchmark FASTQ-to-BAM aDNA workflow",
        stage_ids: &[BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE, BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION],
        scenarios: G162_SCENARIOS,
        checks: ancient_workflow_checks,
    },
    GoalDefinition {
        goal_id: "G163",
        title: "benchmark BAM-to-VCF workflow",
        stage_ids: &[
            BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
            BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G163_SCENARIOS,
        checks: bam_to_vcf_checks,
    },
    GoalDefinition {
        goal_id: "G164",
        title: "benchmark FASTQ-to-VCF workflow",
        stage_ids: &[
            BRIDGE_FASTQ_TRIM_TO_VCF_GL,
            BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G164_SCENARIOS,
        checks: fastq_to_vcf_checks,
    },
];
