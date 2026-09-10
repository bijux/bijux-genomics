//! Owner: hardening campaign planning declarations and pure evidence projections.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use bijux_dna_core::id_catalog as stage;
use std::collections::BTreeSet;

const CAMPAIGN_COVERAGE_PLANNER_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "coverage-planner-stage-tool-boundary",
        focus: "validate campaign coverage planner stage/tool boundary selection",
    },
    ScenarioDefinition {
        scenario_id: "coverage-planner-corpus-db-signal-coverage",
        focus: "validate corpus and database coverage planning signals",
    },
    ScenarioDefinition {
        scenario_id: "coverage-planner-appraisal-proof",
        focus: "capture campaign coverage planner appraisal evidence",
    },
];

const EXPENSIVE_JOB_BUDGET_PLANNER_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "budget-planner-resource-boundary",
        focus: "validate budget planner CPU/memory/storage boundary accounting",
    },
    ScenarioDefinition {
        scenario_id: "budget-planner-encryption-appraiser-signals",
        focus: "validate encrypted output and appraiser cost signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "budget-planner-appraisal-proof",
        focus: "capture budget planner appraisal evidence",
    },
];

const CAMPAIGN_SPLITTING_BY_HPC_QUOTA_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "campaign-split-quota-boundary",
        focus: "validate campaign splitting boundaries by quota/account/partition",
    },
    ScenarioDefinition {
        scenario_id: "campaign-split-priority-window-signals",
        focus: "validate split-window and priority signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "campaign-split-appraisal-proof",
        focus: "capture campaign split appraisal evidence",
    },
];

const CAMPAIGN_REPRODUCIBILITY_LOCK_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "campaign-lock-freeze-boundary",
        focus: "validate campaign reproducibility lock freeze boundaries",
    },
    ScenarioDefinition {
        scenario_id: "campaign-lock-asset-signal-coverage",
        focus: "validate lock coverage signals for corpus, DB, image, and code assets",
    },
    ScenarioDefinition {
        scenario_id: "campaign-lock-appraisal-proof",
        focus: "capture campaign lock appraisal evidence",
    },
];

const CAMPAIGN_RERUN_FROM_LOCK_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "campaign-rerun-lock-replay-boundary",
        focus: "validate rerun-from-lock replay boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "campaign-rerun-diff-signal-coverage",
        focus: "validate rerun diff signal coverage for selected updated assets",
    },
    ScenarioDefinition {
        scenario_id: "campaign-rerun-appraisal-proof",
        focus: "capture campaign rerun appraisal evidence",
    },
];

const CAMPAIGN_SUBSET_FROM_HARDENING_QUEUE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "queue-subset-selector-boundary",
        focus: "validate hardening queue subset selection boundaries",
    },
    ScenarioDefinition {
        scenario_id: "queue-subset-validation-signal-coverage",
        focus: "validate queue subset coverage signals for hardening verification rows",
    },
    ScenarioDefinition {
        scenario_id: "queue-subset-appraisal-proof",
        focus: "capture queue subset appraisal evidence",
    },
];

const CAMPAIGN_PRIORITY_LABELS_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "priority-label-scientific-runtime-boundary",
        focus: "validate campaign priority labels across scientific and runtime boundaries",
    },
    ScenarioDefinition {
        scenario_id: "priority-label-hardening-value-signals",
        focus: "validate priority label signals for expected hardening value",
    },
    ScenarioDefinition {
        scenario_id: "priority-label-appraisal-proof",
        focus: "capture priority label appraisal evidence",
    },
];

const WARM_START_CAMPAIGN_MODE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "warm-start-prepared-asset-boundary",
        focus: "validate warm-start campaign boundary with pre-locked assets",
    },
    ScenarioDefinition {
        scenario_id: "warm-start-preflight-signal-coverage",
        focus: "validate warm-start preflight signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "warm-start-appraisal-proof",
        focus: "capture warm-start campaign appraisal evidence",
    },
];

const ALL_IN_ONE_PREFLIGHT_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "all-in-one-preflight-boundary",
        focus: "validate all-in-one preflight boundaries across required campaign surfaces",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-preflight-failure-signals",
        focus: "validate preflight refusal and warning signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-preflight-appraisal-proof",
        focus: "capture all-in-one preflight appraisal evidence",
    },
];

const ALL_IN_ONE_LOCAL_IMPORT_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "all-in-one-import-decrypt-boundary",
        focus: "validate all-in-one local import/decrypt boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-import-index-signal-coverage",
        focus: "validate local import indexing and appraisal signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-import-appraisal-proof",
        focus: "capture all-in-one local import appraisal evidence",
    },
];

const ALL_IN_ONE_REPORT_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "all-in-one-report-summary-boundary",
        focus: "validate all-in-one report summary boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-report-link-signal-coverage",
        focus: "validate report link coverage signals to detailed artifacts",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-report-appraisal-proof",
        focus: "capture all-in-one report appraisal evidence",
    },
];

const CAMPAIGN_COMPARISON_REPORT_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "campaign-comparison-paired-boundary",
        focus: "validate campaign comparison boundaries across paired campaigns",
    },
    ScenarioDefinition {
        scenario_id: "campaign-comparison-drift-signal-coverage",
        focus: "validate campaign drift and status comparison signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "campaign-comparison-appraisal-proof",
        focus: "capture campaign comparison appraisal evidence",
    },
];

const BENCHMARK_TO_DOCS_UPDATE_HINTS_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "benchmark-doc-hint-boundary",
        focus: "validate benchmark-to-docs hint boundaries for command-impacting behavior",
    },
    ScenarioDefinition {
        scenario_id: "benchmark-doc-hint-caveat-signal-coverage",
        focus: "validate docs hint caveat signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "benchmark-doc-hint-appraisal-proof",
        focus: "capture benchmark-to-docs hint appraisal evidence",
    },
];

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G228",
        title: "add campaign coverage planner",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_COVERAGE, stage::VCF_IMPUTE],
        checks: campaign_coverage_planner,
        scenarios: CAMPAIGN_COVERAGE_PLANNER_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G229",
        title: "add expensive-job budget planner",
        stage_ids: &[stage::BAM_ALIGN, stage::BAM_COVERAGE, stage::VCF_POSTPROCESS],
        checks: expensive_job_budget_planner,
        scenarios: EXPENSIVE_JOB_BUDGET_PLANNER_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G230",
        title: "add campaign splitting by HPC quota",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: campaign_splitting_by_hpc_quota,
        scenarios: CAMPAIGN_SPLITTING_BY_HPC_QUOTA_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G231",
        title: "add campaign reproducibility lock",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_POSTPROCESS],
        checks: campaign_reproducibility_lock,
        scenarios: CAMPAIGN_REPRODUCIBILITY_LOCK_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G232",
        title: "add campaign rerun from lock",
        stage_ids: &[
            stage::FASTQ_STATS_NEUTRAL,
            stage::BAM_MAPPING_SUMMARY,
            stage::VCF_POSTPROCESS,
        ],
        checks: campaign_rerun_from_lock,
        scenarios: CAMPAIGN_RERUN_FROM_LOCK_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G233",
        title: "add campaign subset from hardening queue",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_ALIGN, stage::VCF_FILTER],
        checks: campaign_subset_from_hardening_queue,
        scenarios: CAMPAIGN_SUBSET_FROM_HARDENING_QUEUE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G234",
        title: "add campaign priority labels",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_COVERAGE, stage::VCF_STATS],
        checks: campaign_priority_labels,
        scenarios: CAMPAIGN_PRIORITY_LABELS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G235",
        title: "add warm-start campaign mode",
        stage_ids: &[stage::BAM_VALIDATE, stage::BAM_MAPPING_SUMMARY, stage::VCF_CALL],
        checks: warm_start_campaign_mode,
        scenarios: WARM_START_CAMPAIGN_MODE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G236",
        title: "add all-in-one preflight",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_FILTER],
        checks: all_in_one_preflight,
        scenarios: ALL_IN_ONE_PREFLIGHT_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G237",
        title: "add all-in-one local import",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: all_in_one_local_import,
        scenarios: ALL_IN_ONE_LOCAL_IMPORT_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G238",
        title: "add all-in-one report",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_COVERAGE, stage::VCF_POSTPROCESS],
        checks: all_in_one_report,
        scenarios: ALL_IN_ONE_REPORT_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G239",
        title: "add campaign comparison report",
        stage_ids: &[stage::BAM_ALIGN, stage::VCF_CALL, stage::VCF_STATS],
        checks: campaign_comparison_report,
        scenarios: CAMPAIGN_COMPARISON_REPORT_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G240",
        title: "add benchmark-to-docs update hints",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::VCF_FILTER, stage::VCF_POSTPROCESS],
        checks: benchmark_to_docs_update_hints,
        scenarios: BENCHMARK_TO_DOCS_UPDATE_HINTS_SCENARIOS,
    },
];

fn campaign_coverage_planner(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("campaign_coverage_rows_present={}", !rows.is_empty()),
        format!(
            "campaign_coverage_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "campaign_coverage_profile_coverage_impute_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_IMPUTE)
        ),
        format!(
            "campaign_coverage_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("campaign_coverage_findings_count={}", findings.len()),
    ]
}

fn expensive_job_budget_planner(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("budget_planner_rows_present={}", !rows.is_empty()),
        format!(
            "budget_planner_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "budget_planner_align_coverage_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "budget_planner_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("budget_planner_findings_count={}", findings.len()),
    ]
}

fn campaign_splitting_by_hpc_quota(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let queue_entries = input.queue_entries;
    vec![
        format!("campaign_split_rows_present={}", !rows.is_empty()),
        format!(
            "campaign_split_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "campaign_split_validate_summary_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "campaign_split_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("campaign_split_queue_entries={}", queue_entries.len()),
    ]
}

fn campaign_reproducibility_lock(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("campaign_lock_rows_present={}", !rows.is_empty()),
        format!(
            "campaign_lock_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "campaign_lock_validate_validate_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "campaign_lock_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("campaign_lock_findings_count={}", findings.len()),
    ]
}

fn campaign_rerun_from_lock(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("campaign_rerun_rows_present={}", !rows.is_empty()),
        format!(
            "campaign_rerun_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "campaign_rerun_profile_summary_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "campaign_rerun_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("campaign_rerun_findings_count={}", findings.len()),
    ]
}

fn campaign_subset_from_hardening_queue(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let queue_entries = input.queue_entries;
    vec![
        format!("queue_subset_rows_present={}", !rows.is_empty()),
        format!(
            "queue_subset_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "queue_subset_validate_align_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "queue_subset_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!("queue_subset_queue_entries={}", queue_entries.len()),
    ]
}

fn campaign_priority_labels(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("priority_label_rows_present={}", !rows.is_empty()),
        format!(
            "priority_label_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "priority_label_profile_coverage_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "priority_label_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("priority_label_findings_count={}", findings.len()),
    ]
}

fn warm_start_campaign_mode(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("warm_start_rows_present={}", !rows.is_empty()),
        format!(
            "warm_start_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "warm_start_validate_summary_call_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
        ),
        format!(
            "warm_start_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("warm_start_findings_count={}", findings.len()),
    ]
}

fn all_in_one_preflight(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let queue_entries = input.queue_entries;
    vec![
        format!("all_in_one_preflight_rows_present={}", !rows.is_empty()),
        format!(
            "all_in_one_preflight_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "all_in_one_preflight_validate_validate_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "all_in_one_preflight_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!("all_in_one_preflight_queue_entries={}", queue_entries.len()),
    ]
}

fn all_in_one_local_import(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("all_in_one_import_rows_present={}", !rows.is_empty()),
        format!(
            "all_in_one_import_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "all_in_one_import_profile_summary_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "all_in_one_import_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("all_in_one_import_findings_count={}", findings.len()),
    ]
}

fn all_in_one_report(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("all_in_one_report_rows_present={}", !rows.is_empty()),
        format!(
            "all_in_one_report_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "all_in_one_report_profile_coverage_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "all_in_one_report_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("all_in_one_report_findings_count={}", findings.len()),
    ]
}

fn campaign_comparison_report(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("campaign_comparison_rows_present={}", !rows.is_empty()),
        format!(
            "campaign_comparison_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "campaign_comparison_align_call_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "campaign_comparison_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("campaign_comparison_findings_count={}", findings.len()),
    ]
}

fn benchmark_to_docs_update_hints(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("benchmark_docs_rows_present={}", !rows.is_empty()),
        format!(
            "benchmark_docs_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "benchmark_docs_validate_filter_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "benchmark_docs_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!("benchmark_docs_findings_count={}", findings.len()),
    ]
}
