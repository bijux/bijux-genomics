//! Owner: hardening campaign execution declarations and pure evidence projections.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use bijux_dna_core::id_catalog as stage;
use std::collections::BTreeSet;

const APPTAINER_IMAGE_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "image-smoke-bind-failure-detection",
        focus: "detect image smoke/bind/version failures",
    },
    ScenarioDefinition {
        scenario_id: "image-remediation-plan",
        focus: "generate image-hardening remediation plan",
    },
    ScenarioDefinition {
        scenario_id: "image-rerun-proof",
        focus: "capture rerun proof after image fixes",
    },
];

const SLURM_WRAPPER_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "slurm-wrapper-failure-detection",
        focus: "detect wrapper path/env/signal/output failures",
    },
    ScenarioDefinition {
        scenario_id: "wrapper-remediation-plan",
        focus: "generate wrapper-hardening remediation plan",
    },
    ScenarioDefinition {
        scenario_id: "wrapper-rerun-proof",
        focus: "capture rerun proof for wrapper fixes",
    },
];

const RESOURCE_TUNING_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "resource-drift-detection",
        focus: "detect CPU/mem/walltime/scratch/IO drift",
    },
    ScenarioDefinition {
        scenario_id: "resource-tuning-plan",
        focus: "generate resource tuning remediation plan",
    },
    ScenarioDefinition {
        scenario_id: "resource-before-after-benchmark",
        focus: "record before/after benchmark evidence",
    },
];

const IO_STAGING_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "io-staging-policy-detection",
        focus: "detect scratch/shared-storage IO policy issues",
    },
    ScenarioDefinition {
        scenario_id: "io-staging-remediation-plan",
        focus: "generate IO staging hardening plan",
    },
    ScenarioDefinition {
        scenario_id: "io-improvement-report",
        focus: "capture IO improvement evidence report",
    },
];

const ENCRYPTION_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "encryption-redaction-failure-detection",
        focus: "detect encryption, redaction, key, sidecar failures",
    },
    ScenarioDefinition {
        scenario_id: "security-hardening-plan",
        focus: "generate security hardening plan",
    },
    ScenarioDefinition {
        scenario_id: "security-appraisal-proof",
        focus: "capture security appraisal proof after fixes",
    },
];

const REPLAY_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "replay-field-gap-detection",
        focus: "detect missing replay fields in code/results bundles",
    },
    ScenarioDefinition {
        scenario_id: "replay-hardening-plan",
        focus: "generate replay hardening plan",
    },
    ScenarioDefinition {
        scenario_id: "replay-success-proof",
        focus: "capture replay success evidence",
    },
];

const NONDETERMINISM_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "repeat-run-drift-detection",
        focus: "detect unstable outputs and ordering drift between repeated runs",
    },
    ScenarioDefinition {
        scenario_id: "seed-and-temp-leakage-detection",
        focus: "detect missing seed control and temp-path leakage in artifacts",
    },
    ScenarioDefinition {
        scenario_id: "determinism-rerun-proof",
        focus: "capture rerun proof after determinism hardening",
    },
];

const OUTPUT_SCHEMA_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "schema-parse-failure-detection",
        focus: "detect parser and schema contract failures in result artifacts",
    },
    ScenarioDefinition {
        scenario_id: "schema-contract-fix-queue",
        focus: "queue schema contract hardening entries from appraisal signals",
    },
    ScenarioDefinition {
        scenario_id: "schema-rerun-proof",
        focus: "capture schema-pass proof after hardening",
    },
];

const ACCEPTED_BASELINE_WORKFLOW_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "baseline-promotion-gate-checks",
        focus: "enforce appraisal-pass gates before baseline promotion",
    },
    ScenarioDefinition {
        scenario_id: "baseline-lock-completeness-checks",
        focus: "validate baseline lock completeness for accepted rows",
    },
    ScenarioDefinition {
        scenario_id: "baseline-promotion-proof",
        focus: "capture baseline promotion proof with lock records",
    },
];

const REGRESSION_WORKFLOW_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "baseline-rerun-regression-detection",
        focus: "detect performance/scientific/artifact/replay/security regressions",
    },
    ScenarioDefinition {
        scenario_id: "regression-severity-queue-generation",
        focus: "generate severity-sorted regression hardening queue entries",
    },
    ScenarioDefinition {
        scenario_id: "regression-report-proof",
        focus: "capture regression report evidence from rerun appraisals",
    },
];

const HARDENING_SPRINT_GENERATOR_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sprint-top-risk-selection",
        focus: "select top-risk hardening items for sprint generation",
    },
    ScenarioDefinition {
        scenario_id: "sprint-bookkeeping-exclusion",
        focus: "exclude bookkeeping-only items from sprint backlog",
    },
    ScenarioDefinition {
        scenario_id: "sprint-output-proof",
        focus: "capture sprint output evidence from hardening queue",
    },
];

const CAMPAIGN_SALVAGE_WORKFLOW_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "partial-campaign-evidence-selection",
        focus: "extract valid evidence from partial campaigns",
    },
    ScenarioDefinition {
        scenario_id: "partial-campaign-noncompletion-guard",
        focus: "guard against treating partial campaign as complete",
    },
    ScenarioDefinition {
        scenario_id: "salvage-workflow-proof",
        focus: "capture campaign salvage workflow evidence",
    },
];

const HARDENING_ACCEPTANCE_RERUN_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "acceptance-rerun-anchor-selection",
        focus: "select exact failing and neighboring rows for acceptance rerun",
    },
    ScenarioDefinition {
        scenario_id: "acceptance-rerun-before-after-guard",
        focus: "require before/after evidence for acceptance reruns",
    },
    ScenarioDefinition {
        scenario_id: "acceptance-rerun-proof",
        focus: "capture hardening acceptance rerun evidence",
    },
];

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G175",
        title: "add Apptainer-image hardening",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: apptainer_image_hardening,
        scenarios: APPTAINER_IMAGE_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G176",
        title: "add Slurm-wrapper hardening",
        stage_ids: &[stage::BAM_MAPPING_SUMMARY, stage::BAM_COVERAGE, stage::VCF_STATS],
        checks: slurm_wrapper_hardening,
        scenarios: SLURM_WRAPPER_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G177",
        title: "add resource-tuning hardening",
        stage_ids: &[stage::BAM_ALIGN, stage::BAM_COVERAGE, stage::VCF_CALL],
        checks: resource_tuning_hardening,
        scenarios: RESOURCE_TUNING_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G178",
        title: "add IO-staging hardening",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: io_staging_hardening,
        scenarios: IO_STAGING_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G179",
        title: "add encryption hardening",
        stage_ids: &[stage::BAM_CONTAMINATION, stage::VCF_IMPUTE, stage::VCF_POSTPROCESS],
        checks: encryption_hardening,
        scenarios: ENCRYPTION_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G180",
        title: "add replay hardening",
        stage_ids: &[
            stage::FASTQ_STATS_NEUTRAL,
            stage::BAM_MAPPING_SUMMARY,
            stage::VCF_POSTPROCESS,
        ],
        checks: replay_hardening,
        scenarios: REPLAY_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G181",
        title: "add nondeterminism hardening",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: nondeterminism_hardening,
        scenarios: NONDETERMINISM_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G182",
        title: "add output-schema hardening",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_POSTPROCESS],
        checks: output_schema_hardening,
        scenarios: OUTPUT_SCHEMA_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G189",
        title: "add accepted-baseline workflow",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_STATS],
        checks: accepted_baseline_workflow,
        scenarios: ACCEPTED_BASELINE_WORKFLOW_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G190",
        title: "add regression workflow",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_COVERAGE, stage::VCF_POSTPROCESS],
        checks: regression_workflow,
        scenarios: REGRESSION_WORKFLOW_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G191",
        title: "add hardening sprint generator",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_COVERAGE, stage::VCF_FILTER],
        checks: hardening_sprint_generator,
        scenarios: HARDENING_SPRINT_GENERATOR_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G196",
        title: "add campaign salvage workflow",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_COVERAGE, stage::VCF_FILTER],
        checks: campaign_salvage_workflow,
        scenarios: CAMPAIGN_SALVAGE_WORKFLOW_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G200",
        title: "add hardening acceptance rerun",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_ALIGN, stage::VCF_FILTER],
        checks: hardening_acceptance_rerun,
        scenarios: HARDENING_ACCEPTANCE_RERUN_SCENARIOS,
    },
];

fn apptainer_image_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("image_hardening_rows_present={}", !rows.is_empty()),
        format!(
            "image_hardening_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "image_hardening_mapping_summary_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
        ),
        format!(
            "image_hardening_image_mismatch_findings={}",
            findings.iter().filter(|finding| finding.failure_class == "image-mismatch").count()
        ),
        format!(
            "image_hardening_image_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.failure_class == "image-mismatch").count()
        ),
    ]
}

fn slurm_wrapper_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("wrapper_hardening_rows_present={}", !rows.is_empty()),
        format!(
            "wrapper_hardening_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "wrapper_hardening_summary_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
        ),
        format!(
            "wrapper_hardening_runtime_findings={}",
            findings.iter().filter(|finding| finding.failure_class.starts_with("runtime-")).count()
        ),
        format!(
            "wrapper_hardening_repro_low_repeat_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "reproducibility-low-repeats")
                .count()
        ),
    ]
}

fn resource_tuning_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("resource_hardening_rows_present={}", !rows.is_empty()),
        format!(
            "resource_hardening_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "resource_hardening_degraded_rows={}",
            rows.iter().filter(|row| row.readiness_class == "degraded").count()
        ),
        format!(
            "resource_hardening_min_repetitions={}",
            rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
        ),
        format!(
            "resource_hardening_runtime_under_sampled_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "runtime-under-sampled")
                .count()
        ),
    ]
}

fn io_staging_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("io_hardening_rows_present={}", !rows.is_empty()),
        format!(
            "io_hardening_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "io_hardening_profile_to_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "io_hardening_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!(
            "io_hardening_code_freeze_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "code-freeze-incomplete")
                .count()
        ),
    ]
}

fn encryption_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("security_hardening_rows_present={}", !rows.is_empty()),
        format!(
            "security_hardening_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "security_hardening_encrypted_scope_findings={}",
            findings
                .iter()
                .filter(|finding| finding.result_scope.starts_with("encrypted-"))
                .count()
        ),
        format!(
            "security_hardening_critical_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity == "critical").count()
        ),
        format!(
            "security_hardening_code_freeze_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "code-freeze-incomplete")
                .count()
        ),
    ]
}

fn replay_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("replay_hardening_rows_present={}", !rows.is_empty()),
        format!(
            "replay_hardening_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "replay_hardening_chain_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "replay_hardening_code_freeze_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "code-freeze-incomplete")
                .count()
        ),
        format!(
            "replay_hardening_code_freeze_queue_entries={}",
            queue_entries
                .iter()
                .filter(|entry| entry.source_appraisers.iter().any(|id| id == "code-freeze"))
                .count()
        ),
    ]
}

fn nondeterminism_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("nondeterminism_rows_present={}", !rows.is_empty()),
        format!(
            "nondeterminism_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "nondeterminism_profile_summary_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "nondeterminism_repeat_risk_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!(
            "nondeterminism_repro_runtime_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class.starts_with("runtime-")
                        || finding.failure_class == "reproducibility-low-repeats"
                })
                .count()
        ),
    ]
}

fn output_schema_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("output_schema_rows_present={}", !rows.is_empty()),
        format!(
            "output_schema_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "output_schema_validate_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "output_schema_code_freeze_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "code-freeze-incomplete")
                .count()
        ),
        format!(
            "output_schema_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
    ]
}

fn accepted_baseline_workflow(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("baseline_workflow_rows_present={}", !rows.is_empty()),
        format!(
            "baseline_workflow_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "baseline_workflow_validate_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "baseline_workflow_critical_findings={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
        format!(
            "baseline_workflow_noncritical_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity != "critical").count()
        ),
    ]
}

fn regression_workflow(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("regression_workflow_rows_present={}", !rows.is_empty()),
        format!(
            "regression_workflow_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "regression_workflow_profile_coverage_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "regression_workflow_critical_findings={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
        format!(
            "regression_workflow_non_info_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

fn hardening_sprint_generator(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("sprint_generator_rows_present={}", !rows.is_empty()),
        format!(
            "sprint_generator_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "sprint_generator_high_risk_findings={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
        format!(
            "sprint_generator_non_bookkeeping_entries={}",
            queue_entries
                .iter()
                .filter(|entry| entry.failure_class != "code-freeze-incomplete")
                .count()
        ),
        format!(
            "sprint_generator_profile_coverage_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
    ]
}

fn campaign_salvage_workflow(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("campaign_salvage_rows_present={}", !rows.is_empty()),
        format!(
            "campaign_salvage_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "campaign_salvage_partial_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!(
            "campaign_salvage_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "campaign_salvage_noncritical_findings={}",
            findings.iter().filter(|finding| finding.severity != "critical").count()
        ),
    ]
}

fn hardening_acceptance_rerun(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let queue_entries = input.queue_entries;
    vec![
        format!("acceptance_rerun_rows_present={}", !rows.is_empty()),
        format!(
            "acceptance_rerun_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "acceptance_rerun_validate_align_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "acceptance_rerun_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "acceptance_rerun_before_after_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}
