//! Owner: hardening campaign operations declarations and pure evidence projections.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use std::collections::BTreeSet;

const NIGHTLY_SLURM_MINI_CAMPAIGN_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "nightly-mini-campaign-rotation-boundary",
        focus: "validate nightly mini-campaign rotation boundaries",
    },
    ScenarioDefinition {
        scenario_id: "nightly-mini-campaign-drift-signals",
        focus: "validate nightly drift-history and hardening signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "nightly-mini-campaign-proof",
        focus: "capture nightly mini-campaign evidence",
    },
];

const MONTHLY_FULL_CAMPAIGN_MODE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "monthly-full-campaign-boundary",
        focus: "validate monthly full-campaign baseline comparison boundaries",
    },
    ScenarioDefinition {
        scenario_id: "monthly-full-campaign-regression-signals",
        focus: "validate monthly runtime and scientific regression signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "monthly-full-campaign-proof",
        focus: "capture monthly full-campaign history evidence",
    },
];

const COST_AWARE_CAMPAIGN_PRUNING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "cost-aware-pruning-boundary",
        focus: "validate cost-aware campaign pruning selection boundaries",
    },
    ScenarioDefinition {
        scenario_id: "cost-aware-pruning-value-signals",
        focus: "validate pruning value and coverage tradeoff signals",
    },
    ScenarioDefinition {
        scenario_id: "cost-aware-pruning-proof",
        focus: "capture cost-aware pruning report evidence",
    },
];

const CAMPAIGN_FAILURE_BUDGET_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "failure-budget-threshold-boundary",
        focus: "validate campaign failure budget threshold boundaries",
    },
    ScenarioDefinition {
        scenario_id: "failure-budget-pause-signals",
        focus: "validate failure-rate pause and stop signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "failure-budget-proof",
        focus: "capture failure-budget enforcement evidence",
    },
];

const CAMPAIGN_STORAGE_BUDGET_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "storage-budget-threshold-boundary",
        focus: "validate campaign storage budget threshold boundaries",
    },
    ScenarioDefinition {
        scenario_id: "storage-budget-pause-signals",
        focus: "validate storage exhaustion pause and stop signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "storage-budget-proof",
        focus: "capture storage-budget enforcement evidence",
    },
];

const APPRAISER_HEALTH_BUDGET_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "appraiser-budget-threshold-boundary",
        focus: "validate appraiser health budget threshold boundaries",
    },
    ScenarioDefinition {
        scenario_id: "appraiser-budget-failure-signals",
        focus: "validate appraiser failure and interpretability loss signals",
    },
    ScenarioDefinition {
        scenario_id: "appraiser-budget-proof",
        focus: "capture appraiser health budget evidence",
    },
];

const BENCHMARK_RESULT_RETENTION_TIERS_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "retention-tier-classification-boundary",
        focus: "validate benchmark retention tier classification boundaries",
    },
    ScenarioDefinition {
        scenario_id: "retention-tier-encryption-signals",
        focus: "validate retention tier encryption and purge safety signals",
    },
    ScenarioDefinition {
        scenario_id: "retention-tier-proof",
        focus: "capture retention tier policy evidence",
    },
];

const LOCAL_BENCHMARK_ARCHIVE_FORMAT_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "local-archive-structure-boundary",
        focus: "validate local benchmark archive structure boundaries",
    },
    ScenarioDefinition {
        scenario_id: "local-archive-import-signals",
        focus: "validate archive import and investigation signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "local-archive-proof",
        focus: "capture local benchmark archive evidence",
    },
];

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G269",
        title: "add nightly Slurm mini-campaign",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.filter"],
        checks: nightly_slurm_mini_campaign,
        scenarios: NIGHTLY_SLURM_MINI_CAMPAIGN_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G270",
        title: "add monthly full-campaign mode",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.call"],
        checks: monthly_full_campaign_mode,
        scenarios: MONTHLY_FULL_CAMPAIGN_MODE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G271",
        title: "add cost-aware campaign pruning",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.filter"],
        checks: cost_aware_campaign_pruning,
        scenarios: COST_AWARE_CAMPAIGN_PRUNING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G272",
        title: "add campaign failure budget",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.filter"],
        checks: campaign_failure_budget,
        scenarios: CAMPAIGN_FAILURE_BUDGET_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G273",
        title: "add campaign storage budget",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.postprocess"],
        checks: campaign_storage_budget,
        scenarios: CAMPAIGN_STORAGE_BUDGET_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G275",
        title: "add appraiser health budget",
        stage_ids: &["fastq.validate_reads", "bam.validate", "vcf.stats"],
        checks: appraiser_health_budget,
        scenarios: APPRAISER_HEALTH_BUDGET_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G276",
        title: "add benchmark result retention tiers",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.postprocess"],
        checks: benchmark_result_retention_tiers,
        scenarios: BENCHMARK_RESULT_RETENTION_TIERS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G277",
        title: "add local benchmark archive format",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.postprocess"],
        checks: local_benchmark_archive_format,
        scenarios: LOCAL_BENCHMARK_ARCHIVE_FORMAT_SCENARIOS,
    },
];

fn nightly_slurm_mini_campaign(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("nightly_campaign_rows_present={}", !rows.is_empty()),
        format!(
            "nightly_campaign_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "nightly_campaign_validate_align_filter_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                && rows.iter().any(|row| row.stage_id == "bam.align")
                && rows.iter().any(|row| row.stage_id == "vcf.filter")
        ),
        format!(
            "nightly_campaign_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("nightly_campaign_findings_count={}", findings.len()),
    ]
}

fn monthly_full_campaign_mode(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("monthly_campaign_rows_present={}", !rows.is_empty()),
        format!(
            "monthly_campaign_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "monthly_campaign_profile_coverage_call_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                && rows.iter().any(|row| row.stage_id == "bam.coverage")
                && rows.iter().any(|row| row.stage_id == "vcf.call")
        ),
        format!(
            "monthly_campaign_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("monthly_campaign_findings_count={}", findings.len()),
    ]
}

fn cost_aware_campaign_pruning(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("cost_pruning_rows_present={}", !rows.is_empty()),
        format!(
            "cost_pruning_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "cost_pruning_profile_coverage_filter_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                && rows.iter().any(|row| row.stage_id == "bam.coverage")
                && rows.iter().any(|row| row.stage_id == "vcf.filter")
        ),
        format!(
            "cost_pruning_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("cost_pruning_signal_count={}", findings.len()),
    ]
}

fn campaign_failure_budget(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("failure_budget_rows_present={}", !rows.is_empty()),
        format!(
            "failure_budget_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "failure_budget_validate_align_filter_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                && rows.iter().any(|row| row.stage_id == "bam.align")
                && rows.iter().any(|row| row.stage_id == "vcf.filter")
        ),
        format!(
            "failure_budget_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "failure_budget_triggered={}",
            rows.iter().any(|row| row.readiness_class == "refuse")
                || findings.iter().any(|finding| finding.severity == "critical")
                || queue_entries.iter().any(|entry| entry.severity == "critical")
        ),
    ]
}

fn campaign_storage_budget(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("storage_budget_rows_present={}", !rows.is_empty()),
        format!(
            "storage_budget_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "storage_budget_profile_summary_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
        ),
        format!(
            "storage_budget_runtime_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class == "runtime-outlier"
                        || finding.failure_class == "runtime-under-sampled"
                })
                .count()
        ),
        format!(
            "storage_budget_triggered={}",
            rows.iter().any(|row| row.readiness_class != "ready")
                || findings.iter().any(|finding| finding.failure_class.starts_with("runtime-"))
        ),
    ]
}

fn appraiser_health_budget(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("appraiser_budget_rows_present={}", !rows.is_empty()),
        format!(
            "appraiser_budget_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "appraiser_budget_validate_validate_stats_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                && rows.iter().any(|row| row.stage_id == "bam.validate")
                && rows.iter().any(|row| row.stage_id == "vcf.stats")
        ),
        format!("appraiser_budget_findings_count={}", findings.len()),
        format!(
            "appraiser_budget_triggered={}",
            findings.len() >= 2
                || queue_entries.len() >= 2
                || findings.iter().any(|finding| finding.severity == "critical")
        ),
    ]
}

fn benchmark_result_retention_tiers(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("retention_tier_rows_present={}", !rows.is_empty()),
        format!(
            "retention_tier_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "retention_tier_profile_summary_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
        ),
        format!(
            "retention_tier_encrypted_findings={}",
            findings
                .iter()
                .filter(|finding| finding.result_scope.starts_with("encrypted-"))
                .count()
        ),
        format!(
            "retention_tier_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
    ]
}

fn local_benchmark_archive_format(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("local_archive_rows_present={}", !rows.is_empty()),
        format!(
            "local_archive_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "local_archive_profile_coverage_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                && rows.iter().any(|row| row.stage_id == "bam.coverage")
                && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
        ),
        format!(
            "local_archive_encrypted_findings={}",
            findings
                .iter()
                .filter(|finding| finding.result_scope.starts_with("encrypted-"))
                .count()
        ),
        format!(
            "local_archive_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
    ]
}
