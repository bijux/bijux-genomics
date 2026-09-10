//! Owner: hardening campaign security declarations and pure evidence projections.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use bijux_dna_core::id_catalog as stage;
use std::collections::BTreeSet;

const ENCRYPTED_BENCHMARK_SHARING_PACKAGE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "encrypted-sharing-package-boundary",
        focus: "validate encrypted sharing package row and scope boundaries",
    },
    ScenarioDefinition {
        scenario_id: "encrypted-sharing-recipient-signals",
        focus: "validate recipient coverage and encrypted scope signals",
    },
    ScenarioDefinition {
        scenario_id: "encrypted-sharing-package-proof",
        focus: "capture encrypted package and sidecar sharing evidence",
    },
];

const REVIEWER_DECRYPT_PROFILE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "reviewer-decrypt-scope-boundary",
        focus: "validate reviewer decrypt profile scope and authorization boundaries",
    },
    ScenarioDefinition {
        scenario_id: "reviewer-decrypt-recipient-signals",
        focus: "validate recipient-scoped decrypt findings and queue signals",
    },
    ScenarioDefinition {
        scenario_id: "reviewer-decrypt-proof",
        focus: "capture reviewer decrypt profile evidence",
    },
];

const CONFIDENTIAL_ISSUE_BUNDLE_GENERATOR_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "confidential-issue-bundle-boundary",
        focus: "validate confidential issue bundle minimization boundaries",
    },
    ScenarioDefinition {
        scenario_id: "confidential-issue-redaction-signals",
        focus: "validate confidential redaction and encrypted scope signals",
    },
    ScenarioDefinition {
        scenario_id: "confidential-issue-bundle-proof",
        focus: "capture confidential issue bundle generation evidence",
    },
];

const SOURCE_BLAME_FROM_CODE_BUNDLES_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "source-blame-change-boundary",
        focus: "validate source blame change-surface attribution boundaries",
    },
    ScenarioDefinition {
        scenario_id: "source-blame-drift-signals",
        focus: "validate source blame drift and mismatch signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "source-blame-proof",
        focus: "capture source blame report evidence",
    },
];

const PERFORMANCE_REGRESSION_GATE_FROM_SLURM_BASELINES_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "performance-regression-gate-boundary",
        focus: "validate performance regression gate baseline boundaries",
    },
    ScenarioDefinition {
        scenario_id: "performance-regression-runtime-signals",
        focus: "validate runtime and resource regression signals",
    },
    ScenarioDefinition {
        scenario_id: "performance-regression-proof",
        focus: "capture performance regression gate evidence",
    },
];

const SCIENTIFIC_REGRESSION_GATE_FROM_SLURM_BASELINES_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "scientific-regression-gate-boundary",
        focus: "validate scientific regression gate baseline boundaries",
    },
    ScenarioDefinition {
        scenario_id: "scientific-regression-caveat-signals",
        focus: "validate scientific output and caveat regression signals",
    },
    ScenarioDefinition {
        scenario_id: "scientific-regression-proof",
        focus: "capture scientific regression gate evidence",
    },
];

const SECURITY_REGRESSION_GATE_FROM_SLURM_BASELINES_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "security-regression-gate-boundary",
        focus: "validate security regression gate and encryption boundaries",
    },
    ScenarioDefinition {
        scenario_id: "security-regression-leakage-signals",
        focus: "validate leakage, redaction, and encryption failure signals",
    },
    ScenarioDefinition {
        scenario_id: "security-regression-proof",
        focus: "capture security regression gate evidence",
    },
];

const REPLAY_REGRESSION_GATE_FROM_CODE_RESULTS_BUNDLES_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "replay-regression-gate-boundary",
        focus: "validate replay regression gate reconstructability boundaries",
    },
    ScenarioDefinition {
        scenario_id: "replay-regression-code-result-signals",
        focus: "validate replay regression signals from code and results bundles",
    },
    ScenarioDefinition {
        scenario_id: "replay-regression-proof",
        focus: "capture replay regression gate evidence",
    },
];

const ENCRYPTION_HEALTH_BUDGET_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "encryption-budget-threshold-boundary",
        focus: "validate encryption health budget threshold boundaries",
    },
    ScenarioDefinition {
        scenario_id: "encryption-budget-failure-signals",
        focus: "validate encryption failure and sidecar health signals",
    },
    ScenarioDefinition {
        scenario_id: "encryption-budget-proof",
        focus: "capture encryption health budget evidence",
    },
];

const BENCHMARK_RE_ENCRYPTION_FOR_ARCHIVAL_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "benchmark-reencryption-boundary",
        focus: "validate benchmark re-encryption and key rotation boundaries",
    },
    ScenarioDefinition {
        scenario_id: "benchmark-reencryption-health-signals",
        focus: "validate re-encryption health and decryptability signals",
    },
    ScenarioDefinition {
        scenario_id: "benchmark-reencryption-proof",
        focus: "capture benchmark re-encryption evidence",
    },
];

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G261",
        title: "add encrypted benchmark sharing package",
        stage_ids: &[
            stage::FASTQ_STATS_NEUTRAL,
            stage::BAM_MAPPING_SUMMARY,
            stage::VCF_POSTPROCESS,
        ],
        checks: encrypted_benchmark_sharing_package,
        scenarios: ENCRYPTED_BENCHMARK_SHARING_PACKAGE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G262",
        title: "add reviewer decrypt profile",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_POSTPROCESS],
        checks: reviewer_decrypt_profile,
        scenarios: REVIEWER_DECRYPT_PROFILE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G263",
        title: "add confidential issue bundle generator",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_CONTAMINATION, stage::VCF_FILTER],
        checks: confidential_issue_bundle_generator,
        scenarios: CONFIDENTIAL_ISSUE_BUNDLE_GENERATOR_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G264",
        title: "add source blame from code bundles",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_ALIGN, stage::VCF_POSTPROCESS],
        checks: source_blame_from_code_bundles,
        scenarios: SOURCE_BLAME_FROM_CODE_BUNDLES_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G265",
        title: "add performance-regression gate from Slurm baselines",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_COVERAGE, stage::VCF_STATS],
        checks: performance_regression_gate_from_slurm_baselines,
        scenarios: PERFORMANCE_REGRESSION_GATE_FROM_SLURM_BASELINES_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G266",
        title: "add scientific-regression gate from Slurm baselines",
        stage_ids: &[stage::BAM_MAPPING_SUMMARY, stage::VCF_FILTER, stage::VCF_STATS],
        checks: scientific_regression_gate_from_slurm_baselines,
        scenarios: SCIENTIFIC_REGRESSION_GATE_FROM_SLURM_BASELINES_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G267",
        title: "add security-regression gate from Slurm baselines",
        stage_ids: &[stage::BAM_CONTAMINATION, stage::VCF_IMPUTE, stage::VCF_POSTPROCESS],
        checks: security_regression_gate_from_slurm_baselines,
        scenarios: SECURITY_REGRESSION_GATE_FROM_SLURM_BASELINES_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G268",
        title: "add replay-regression gate from code/results bundles",
        stage_ids: &[
            stage::FASTQ_STATS_NEUTRAL,
            stage::BAM_MAPPING_SUMMARY,
            stage::VCF_POSTPROCESS,
        ],
        checks: replay_regression_gate_from_code_results_bundles,
        scenarios: REPLAY_REGRESSION_GATE_FROM_CODE_RESULTS_BUNDLES_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G274",
        title: "add encryption health budget",
        stage_ids: &[stage::BAM_CONTAMINATION, stage::VCF_IMPUTE, stage::VCF_POSTPROCESS],
        checks: encryption_health_budget,
        scenarios: ENCRYPTION_HEALTH_BUDGET_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G278",
        title: "add benchmark re-encryption for archival",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_POSTPROCESS],
        checks: benchmark_re_encryption_for_archival,
        scenarios: BENCHMARK_RE_ENCRYPTION_FOR_ARCHIVAL_SCENARIOS,
    },
];

fn encrypted_benchmark_sharing_package(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("encrypted_sharing_rows_present={}", !rows.is_empty()),
        format!(
            "encrypted_sharing_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "encrypted_sharing_profile_summary_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "encrypted_sharing_code_and_results_findings={}",
            findings
                .iter()
                .filter(|finding| finding.result_scope == "encrypted-code"
                    || finding.result_scope == "encrypted-results")
                .count()
        ),
        format!("encrypted_sharing_queue_entries={}", queue_entries.len()),
    ]
}

fn reviewer_decrypt_profile(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("reviewer_decrypt_rows_present={}", !rows.is_empty()),
        format!(
            "reviewer_decrypt_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "reviewer_decrypt_validate_validate_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "reviewer_decrypt_encrypted_findings={}",
            findings
                .iter()
                .filter(|finding| finding.result_scope.starts_with("encrypted-"))
                .count()
        ),
        format!(
            "reviewer_decrypt_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
    ]
}

fn confidential_issue_bundle_generator(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("confidential_bundle_rows_present={}", !rows.is_empty()),
        format!(
            "confidential_bundle_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "confidential_bundle_trim_contamination_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::BAM_CONTAMINATION)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "confidential_bundle_encrypted_findings={}",
            findings
                .iter()
                .filter(|finding| finding.result_scope.starts_with("encrypted-"))
                .count()
        ),
        format!(
            "confidential_bundle_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
    ]
}

fn source_blame_from_code_bundles(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("source_blame_rows_present={}", !rows.is_empty()),
        format!(
            "source_blame_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "source_blame_profile_align_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "source_blame_drift_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class == "database-mismatch"
                        || finding.failure_class == "image-mismatch"
                        || finding.failure_class == "corpus-mismatch"
                })
                .count()
        ),
        format!(
            "source_blame_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
    ]
}

fn performance_regression_gate_from_slurm_baselines(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("performance_regression_rows_present={}", !rows.is_empty()),
        format!(
            "performance_regression_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "performance_regression_trim_coverage_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "performance_regression_runtime_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class == "runtime-under-sampled"
                        || finding.failure_class == "runtime-outlier"
                })
                .count()
        ),
        format!(
            "performance_regression_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
    ]
}

fn scientific_regression_gate_from_slurm_baselines(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("scientific_regression_rows_present={}", !rows.is_empty()),
        format!(
            "scientific_regression_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "scientific_regression_summary_filter_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "scientific_regression_scientific_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "scientific-invalidity")
                .count()
        ),
        format!(
            "scientific_regression_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
    ]
}

fn security_regression_gate_from_slurm_baselines(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("security_regression_rows_present={}", !rows.is_empty()),
        format!(
            "security_regression_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "security_regression_contamination_impute_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_CONTAMINATION)
                && rows.iter().any(|row| row.stage_id == stage::VCF_IMPUTE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "security_regression_encrypted_findings={}",
            findings
                .iter()
                .filter(|finding| finding.result_scope.starts_with("encrypted-"))
                .count()
        ),
        format!(
            "security_regression_critical_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity == "critical").count()
        ),
    ]
}

fn replay_regression_gate_from_code_results_bundles(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("replay_regression_rows_present={}", !rows.is_empty()),
        format!(
            "replay_regression_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "replay_regression_profile_summary_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "replay_regression_code_freeze_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "code-freeze-incomplete")
                .count()
        ),
        format!(
            "replay_regression_code_freeze_queue_entries={}",
            queue_entries
                .iter()
                .filter(|entry| entry.source_appraisers.iter().any(|id| id == "code-freeze"))
                .count()
        ),
    ]
}

fn encryption_health_budget(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("encryption_budget_rows_present={}", !rows.is_empty()),
        format!(
            "encryption_budget_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "encryption_budget_contamination_impute_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_CONTAMINATION)
                && rows.iter().any(|row| row.stage_id == stage::VCF_IMPUTE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "encryption_budget_encrypted_findings={}",
            findings
                .iter()
                .filter(|finding| finding.result_scope.starts_with("encrypted-"))
                .count()
        ),
        format!(
            "encryption_budget_critical_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity == "critical").count()
        ),
    ]
}

fn benchmark_re_encryption_for_archival(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("reencryption_rows_present={}", !rows.is_empty()),
        format!(
            "reencryption_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "reencryption_validate_validate_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "reencryption_encrypted_findings={}",
            findings
                .iter()
                .filter(|finding| finding.result_scope.starts_with("encrypted-"))
                .count()
        ),
        format!(
            "reencryption_gate_triggered={}",
            findings.iter().any(|finding| finding.result_scope.starts_with("encrypted-"))
                || queue_entries.iter().any(|entry| entry.severity == "critical")
        ),
    ]
}
