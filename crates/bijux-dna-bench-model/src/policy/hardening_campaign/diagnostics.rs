//! Owner: hardening campaign diagnostics declarations and pure evidence projections.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use bijux_dna_core::id_catalog as stage;
use std::collections::BTreeSet;

const PER_STAGE_HARDENING_PLAYBOOKS_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "per-stage-failure-to-playbook",
        focus: "convert stage failures into concrete hardening playbooks",
    },
    ScenarioDefinition {
        scenario_id: "stage-rerun-proof-path",
        focus: "capture rerun proof path per failed stage",
    },
    ScenarioDefinition {
        scenario_id: "stage-hardening-queue-export",
        focus: "export stage-specific hardening queue views",
    },
];

const PER_TOOL_HARDENING_PLAYBOOKS_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "tool-version-grouping",
        focus: "group failures by tool and version",
    },
    ScenarioDefinition {
        scenario_id: "tool-specific-hardening-queue",
        focus: "emit tool-specific hardening queue items",
    },
    ScenarioDefinition {
        scenario_id: "tool-rerun-proof",
        focus: "attach rerun proof for tool-level hardening",
    },
];

const CAVEAT_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "advisory-caveat-coverage",
        focus: "detect missing or over-promoted advisory caveats",
    },
    ScenarioDefinition {
        scenario_id: "scientific-caveat-correction",
        focus: "generate caveat correction plan from scientific findings",
    },
    ScenarioDefinition {
        scenario_id: "caveat-appraiser-pass",
        focus: "capture caveat appraiser pass evidence after fixes",
    },
];

const FAILURE_TAXONOMY_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "taxonomy-classification-gaps",
        focus: "detect failure classification gaps across runtime/tool/io/scientific classes",
    },
    ScenarioDefinition {
        scenario_id: "taxonomy-grouping-hardening",
        focus: "harden grouping quality for actionable hardening queues",
    },
    ScenarioDefinition {
        scenario_id: "taxonomy-report-quality-proof",
        focus: "capture evidence that grouped reports improved after hardening",
    },
];

const BENCHMARK_MINIMIZER_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "minimizer-failure-anchor-detection",
        focus: "anchor reproducible failure rows for minimization candidates",
    },
    ScenarioDefinition {
        scenario_id: "minimized-input-plan",
        focus: "generate minimal corpus/config/database reproduction plans",
    },
    ScenarioDefinition {
        scenario_id: "minimized-bundle-proof",
        focus: "capture minimized encrypted failure bundle evidence",
    },
];

const LOCAL_INVESTIGATION_WORKSPACE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "local-investigation-layout-validation",
        focus: "validate local investigation workspace structure for copied bundles",
    },
    ScenarioDefinition {
        scenario_id: "local-index-and-catalog-checks",
        focus: "validate indexing and organization checks for local investigation",
    },
    ScenarioDefinition {
        scenario_id: "investigation-workflow-proof",
        focus: "capture local investigation workflow proof from hardening checks",
    },
];

const RESULT_DIFF_WORKFLOW_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "result-metric-diff-detection",
        focus: "detect changed metrics and caveats across result bundles",
    },
    ScenarioDefinition {
        scenario_id: "result-artifact-diff-detection",
        focus: "detect changed artifacts/logs/appraisals across result bundles",
    },
    ScenarioDefinition {
        scenario_id: "result-diff-proof",
        focus: "capture evidence that result diff workflow is actionable",
    },
];

const CODE_DIFF_WORKFLOW_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "code-snapshot-diff-detection",
        focus: "detect source/config/dag drift between code bundles",
    },
    ScenarioDefinition {
        scenario_id: "image-db-corpus-diff-detection",
        focus: "detect image/database/corpus lock drift between code bundles",
    },
    ScenarioDefinition {
        scenario_id: "code-diff-proof",
        focus: "capture evidence that code diff workflow is actionable",
    },
];

const AMBIGUOUS_RESULT_REVIEW_QUEUE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "ambiguous-finding-detection",
        focus: "detect weak or conflicting findings for manual review",
    },
    ScenarioDefinition {
        scenario_id: "manual-review-routing",
        focus: "route ambiguous findings into manual review queue",
    },
    ScenarioDefinition {
        scenario_id: "review-queue-proof",
        focus: "capture manual review queue evidence",
    },
];

const SUSPECTED_UPSTREAM_TOOL_ISSUE_BUNDLE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "upstream-tool-suspect-detection",
        focus: "detect suspected upstream tool behavior from appraisals",
    },
    ScenarioDefinition {
        scenario_id: "upstream-tool-bundle-selection",
        focus: "select minimal rows for encrypted upstream tool issue bundles",
    },
    ScenarioDefinition {
        scenario_id: "upstream-tool-bundle-proof",
        focus: "capture issue bundle readiness proof for upstream reports",
    },
];

const SUSPECTED_DATA_ISSUE_BUNDLE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "data-issue-suspect-detection",
        focus: "detect suspected corpus or database issue findings",
    },
    ScenarioDefinition {
        scenario_id: "data-issue-bundle-selection",
        focus: "select minimal rows for encrypted data issue bundles",
    },
    ScenarioDefinition {
        scenario_id: "data-issue-bundle-proof",
        focus: "capture data issue bundle readiness proof",
    },
];

const SUSPECTED_RUNTIME_ISSUE_BUNDLE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "runtime-issue-suspect-detection",
        focus: "detect runtime and wrapper issue findings",
    },
    ScenarioDefinition {
        scenario_id: "runtime-issue-bundle-selection",
        focus: "select minimal rows for encrypted runtime issue bundles",
    },
    ScenarioDefinition {
        scenario_id: "runtime-issue-bundle-proof",
        focus: "capture runtime issue bundle readiness proof",
    },
];

const APPRAISER_CONFIDENCE_SCORES_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "confidence-band-detection",
        focus: "label findings into strong, weak, and manual-review-required bands",
    },
    ScenarioDefinition {
        scenario_id: "confidence-score-routing",
        focus: "route weak findings to review while preserving strong findings",
    },
    ScenarioDefinition {
        scenario_id: "confidence-proof",
        focus: "capture confidence scoring evidence in hardening outputs",
    },
];

const BENCHMARK_HISTORY_ANALYTICS_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "history-trend-anchor-selection",
        focus: "anchor stage/tool/corpus/db/image/runtime trend evidence",
    },
    ScenarioDefinition {
        scenario_id: "history-analytics-signal-coverage",
        focus: "cover degraded and critical trend signals for analytics",
    },
    ScenarioDefinition {
        scenario_id: "history-report-proof",
        focus: "capture benchmark history analytics report evidence",
    },
];

const STAGE_RISK_RANKING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "stage-risk-severity-selection",
        focus: "rank stage risk by severity and failure frequency",
    },
    ScenarioDefinition {
        scenario_id: "stage-risk-priority-ordering",
        focus: "order stage hardening priorities from risk signals",
    },
    ScenarioDefinition {
        scenario_id: "stage-risk-proof",
        focus: "capture stage-risk ranking report evidence",
    },
];

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G171",
        title: "add per-stage hardening playbooks",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_FILTER],
        checks: per_stage_hardening_playbooks,
        scenarios: PER_STAGE_HARDENING_PLAYBOOKS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G172",
        title: "add per-tool hardening playbooks",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_ALIGN, stage::VCF_CALL],
        checks: per_tool_hardening_playbooks,
        scenarios: PER_TOOL_HARDENING_PLAYBOOKS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G183",
        title: "add caveat hardening",
        stage_ids: &[stage::BAM_CONTAMINATION, stage::VCF_FILTER, stage::VCF_STATS],
        checks: caveat_hardening,
        scenarios: CAVEAT_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G184",
        title: "add failure taxonomy hardening",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_ALIGN, stage::VCF_FILTER],
        checks: failure_taxonomy_hardening,
        scenarios: FAILURE_TAXONOMY_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G185",
        title: "add benchmark minimizer",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_ALIGN, stage::VCF_FILTER],
        checks: benchmark_minimizer,
        scenarios: BENCHMARK_MINIMIZER_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G186",
        title: "add local investigation workspace",
        stage_ids: &[
            stage::FASTQ_STATS_NEUTRAL,
            stage::BAM_MAPPING_SUMMARY,
            stage::VCF_POSTPROCESS,
        ],
        checks: local_investigation_workspace,
        scenarios: LOCAL_INVESTIGATION_WORKSPACE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G187",
        title: "add result diff workflow",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_COVERAGE, stage::VCF_STATS],
        checks: result_diff_workflow,
        scenarios: RESULT_DIFF_WORKFLOW_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G188",
        title: "add code diff workflow",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_MAPPING_SUMMARY, stage::VCF_POSTPROCESS],
        checks: code_diff_workflow,
        scenarios: CODE_DIFF_WORKFLOW_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G192",
        title: "add ambiguous-result review queue",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_ALIGN, stage::VCF_FILTER],
        checks: ambiguous_result_review_queue,
        scenarios: AMBIGUOUS_RESULT_REVIEW_QUEUE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G193",
        title: "add suspected upstream tool issue bundle",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_ALIGN, stage::VCF_CALL],
        checks: suspected_upstream_tool_issue_bundle,
        scenarios: SUSPECTED_UPSTREAM_TOOL_ISSUE_BUNDLE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G194",
        title: "add suspected data issue bundle",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_CONTAMINATION, stage::VCF_FILTER],
        checks: suspected_data_issue_bundle,
        scenarios: SUSPECTED_DATA_ISSUE_BUNDLE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G195",
        title: "add suspected runtime issue bundle",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: suspected_runtime_issue_bundle,
        scenarios: SUSPECTED_RUNTIME_ISSUE_BUNDLE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G197",
        title: "add appraiser confidence scores",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_STATS],
        checks: appraiser_confidence_scores,
        scenarios: APPRAISER_CONFIDENCE_SCORES_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G198",
        title: "add benchmark history analytics",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_COVERAGE, stage::VCF_IMPUTE],
        checks: benchmark_history_analytics,
        scenarios: BENCHMARK_HISTORY_ANALYTICS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G199",
        title: "add stage-risk ranking",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_COVERAGE, stage::VCF_FILTER],
        checks: stage_risk_ranking,
        scenarios: STAGE_RISK_RANKING_SCENARIOS,
    },
];

fn per_stage_hardening_playbooks(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let queue_entries = input.queue_entries;
    vec![
        format!("stage_playbook_rows_present={}", !rows.is_empty()),
        format!(
            "stage_playbook_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "stage_playbook_validate_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
        ),
        format!(
            "stage_playbook_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "stage_playbook_non_info_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

fn per_tool_hardening_playbooks(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("tool_playbook_rows_present={}", !rows.is_empty()),
        format!(
            "tool_playbook_tool_count={}",
            rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "tool_playbook_degraded_or_refuse_tools={}",
            rows.iter()
                .filter(|row| row.readiness_class != "ready")
                .map(|row| row.tool_id.clone())
                .collect::<BTreeSet<_>>()
                .len()
        ),
        format!(
            "tool_playbook_tool_binding_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "missing-tool-binding")
                .count()
        ),
        format!(
            "tool_playbook_queue_non_info={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

fn caveat_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("caveat_hardening_rows_present={}", !rows.is_empty()),
        format!(
            "caveat_hardening_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "caveat_hardening_contam_filter_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_CONTAMINATION)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "caveat_hardening_scientific_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "scientific-invalidity")
                .count()
        ),
        format!(
            "caveat_hardening_corpus_findings={}",
            findings.iter().filter(|finding| finding.failure_class == "corpus-mismatch").count()
        ),
    ]
}

fn failure_taxonomy_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("failure_taxonomy_rows_present={}", !rows.is_empty()),
        format!(
            "failure_taxonomy_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "failure_taxonomy_class_count={}",
            findings
                .iter()
                .map(|finding| finding.failure_class.clone())
                .collect::<BTreeSet<_>>()
                .len()
        ),
        format!(
            "failure_taxonomy_runtime_tool_science_classes={}",
            findings.iter().any(|finding| finding.failure_class.starts_with("runtime-"))
                && findings.iter().any(|finding| finding.failure_class == "missing-tool-binding")
                && findings.iter().any(|finding| finding.failure_class == "scientific-invalidity")
        ),
        format!(
            "failure_taxonomy_queue_class_count={}",
            queue_entries
                .iter()
                .map(|entry| entry.failure_class.clone())
                .collect::<BTreeSet<_>>()
                .len()
        ),
    ]
}

fn benchmark_minimizer(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("benchmark_minimizer_rows_present={}", !rows.is_empty()),
        format!(
            "benchmark_minimizer_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "benchmark_minimizer_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "benchmark_minimizer_critical_findings={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
        format!(
            "benchmark_minimizer_critical_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity == "critical").count()
        ),
    ]
}

fn local_investigation_workspace(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("investigation_workspace_rows_present={}", !rows.is_empty()),
        format!(
            "investigation_workspace_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "investigation_workspace_profile_summary_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "investigation_workspace_code_scope_findings={}",
            findings.iter().filter(|finding| finding.result_scope == "encrypted-code").count()
        ),
        format!("investigation_workspace_queue_entries={}", queue_entries.len()),
    ]
}

fn result_diff_workflow(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("result_diff_rows_present={}", !rows.is_empty()),
        format!(
            "result_diff_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "result_diff_profile_coverage_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "result_diff_encrypted_result_findings={}",
            findings.iter().filter(|finding| finding.result_scope == "encrypted-results").count()
        ),
        format!(
            "result_diff_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
    ]
}

fn code_diff_workflow(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("code_diff_rows_present={}", !rows.is_empty()),
        format!(
            "code_diff_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "code_diff_trim_summary_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "code_diff_code_scope_findings={}",
            findings.iter().filter(|finding| finding.result_scope == "encrypted-code").count()
        ),
        format!(
            "code_diff_code_freeze_queue_entries={}",
            queue_entries
                .iter()
                .filter(|entry| entry.source_appraisers.iter().any(|id| id == "code-freeze"))
                .count()
        ),
    ]
}

fn ambiguous_result_review_queue(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("ambiguous_review_rows_present={}", !rows.is_empty()),
        format!(
            "ambiguous_review_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "ambiguous_review_degraded_rows={}",
            rows.iter().filter(|row| row.readiness_class == "degraded").count()
        ),
        format!(
            "ambiguous_review_warning_findings={}",
            findings.iter().filter(|finding| finding.severity == "warning").count()
        ),
        format!(
            "ambiguous_review_queue_warning_entries={}",
            queue_entries.iter().filter(|entry| entry.severity == "warning").count()
        ),
    ]
}

fn suspected_upstream_tool_issue_bundle(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("upstream_tool_bundle_rows_present={}", !rows.is_empty()),
        format!(
            "upstream_tool_bundle_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "upstream_tool_bundle_trim_align_call_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
        ),
        format!(
            "upstream_tool_bundle_tool_binding_findings={}",
            findings
                .iter()
                .filter(|finding| finding.failure_class == "missing-tool-binding")
                .count()
        ),
        format!(
            "upstream_tool_bundle_queue_non_info={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

fn suspected_data_issue_bundle(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("data_issue_bundle_rows_present={}", !rows.is_empty()),
        format!(
            "data_issue_bundle_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "data_issue_bundle_validate_contam_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_CONTAMINATION)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "data_issue_bundle_corpus_findings={}",
            findings.iter().filter(|finding| finding.failure_class == "corpus-mismatch").count()
        ),
        format!(
            "data_issue_bundle_security_queue_entries={}",
            queue_entries
                .iter()
                .filter(|entry| entry.failure_class.contains("readiness")
                    || entry.failure_class == "corpus-mismatch")
                .count()
        ),
    ]
}

fn suspected_runtime_issue_bundle(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("runtime_issue_bundle_rows_present={}", !rows.is_empty()),
        format!(
            "runtime_issue_bundle_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "runtime_issue_bundle_profile_summary_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "runtime_issue_bundle_runtime_findings={}",
            findings.iter().filter(|finding| finding.failure_class.starts_with("runtime-")).count()
        ),
        format!(
            "runtime_issue_bundle_runtime_queue_entries={}",
            queue_entries
                .iter()
                .filter(|entry| entry.failure_class.starts_with("runtime-"))
                .count()
        ),
    ]
}

fn appraiser_confidence_scores(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("confidence_scores_rows_present={}", !rows.is_empty()),
        format!(
            "confidence_scores_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "confidence_scores_high_confidence_findings={}",
            findings.iter().filter(|finding| finding.confidence == "high").count()
        ),
        format!(
            "confidence_scores_medium_confidence_findings={}",
            findings.iter().filter(|finding| finding.confidence == "medium").count()
        ),
        format!(
            "confidence_scores_manual_review_proxy={}",
            findings
                .iter()
                .any(|finding| finding.confidence == "medium" && finding.severity == "warning")
        ),
    ]
}

fn benchmark_history_analytics(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("history_analytics_rows_present={}", !rows.is_empty()),
        format!(
            "history_analytics_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "history_analytics_profile_coverage_impute_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_IMPUTE)
        ),
        format!(
            "history_analytics_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
        format!(
            "history_analytics_unique_failure_classes={}",
            findings
                .iter()
                .map(|finding| finding.failure_class.clone())
                .collect::<BTreeSet<_>>()
                .len()
        ),
    ]
}

fn stage_risk_ranking(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("stage_risk_ranking_rows_present={}", !rows.is_empty()),
        format!(
            "stage_risk_ranking_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "stage_risk_ranking_trim_coverage_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "stage_risk_ranking_critical_findings={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
        format!(
            "stage_risk_ranking_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
    ]
}
