//! Owner: original hardening diagnostics regression scenarios.
use super::*;

#[test]
fn emits_stage_playbook_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G171".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0101".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        affected_rows: vec!["h10".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("stage_playbook_stage_count=3")));
    assert!(entries[0].goal_checks.iter().any(|check| check == "stage_playbook_refuse_rows=1"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "stage_playbook_non_info_queue_entries=1"));
}

#[test]
fn emits_tool_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G172".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "artifact-validity".to_string(),
        row_id: "h5".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "missing-tool-binding".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "tool binding missing".to_string(),
        recommendation: "bind stage to at least one governed tool".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0002".to_string(),
        severity: "warning".to_string(),
        failure_class: "missing-tool-binding".to_string(),
        recommendation: "bind stage to at least one governed tool".to_string(),
        affected_rows: vec!["h5".to_string()],
        source_appraisers: vec!["artifact-validity".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].goal_checks.iter().any(|check| check == "tool_playbook_tool_count=1"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "tool_playbook_tool_binding_findings=1"));
}

#[test]
fn emits_caveat_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G183".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "scientific-invalidity".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "invalid scientific readiness for caveat outputs".to_string(),
        recommendation: "resolve readiness mismatches before scientific evaluation".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "caveat_hardening_contam_filter_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "caveat_hardening_scientific_findings=1"));
}

#[test]
fn emits_failure_taxonomy_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G184".to_string()];
    let findings = vec![
        AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "h5".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "runtime under sampled".to_string(),
            recommendation: "increase repetitions to at least 2".to_string(),
        },
        AppraisalFinding {
            appraiser_id: "artifact-validity".to_string(),
            row_id: "h5".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "missing-tool-binding".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "missing tool binding".to_string(),
            recommendation: "bind stage to at least one governed tool".to_string(),
        },
        AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "scientific-invalidity".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "scientific invalidity".to_string(),
            recommendation: "resolve readiness mismatches before scientific evaluation".to_string(),
        },
    ];
    let queue = vec![
        HardeningQueueEntry {
            queue_id: "hardening-0201".to_string(),
            severity: "critical".to_string(),
            failure_class: "missing-tool-binding".to_string(),
            recommendation: "bind stage to at least one governed tool".to_string(),
            affected_rows: vec!["h5".to_string()],
            source_appraisers: vec!["artifact-validity".to_string()],
        },
        HardeningQueueEntry {
            queue_id: "hardening-0202".to_string(),
            severity: "warning".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            recommendation: "increase repetitions to at least 2".to_string(),
            affected_rows: vec!["h5".to_string()],
            source_appraisers: vec!["runtime-performance".to_string()],
        },
    ];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "failure_taxonomy_runtime_tool_science_classes=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "failure_taxonomy_queue_class_count=2"));
}

#[test]
fn emits_benchmark_minimizer_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G185".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "refuse row available for minimizer anchoring".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0301".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        affected_rows: vec!["h10".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "benchmark_minimizer_refuse_rows=1"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "benchmark_minimizer_critical_queue_entries=1"));
}

#[test]
fn emits_local_investigation_workspace_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G186".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "local workspace indexing missing code lock metadata".to_string(),
        recommendation: "bind tool and image lock before code freeze publication".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0401".to_string(),
        severity: "critical".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        recommendation: "bind tool and image lock before code freeze publication".to_string(),
        affected_rows: vec!["h13".to_string()],
        source_appraisers: vec!["code-freeze".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "investigation_workspace_profile_summary_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "investigation_workspace_code_scope_findings=1"));
}

#[test]
fn emits_result_diff_workflow_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G187".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h7".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "readiness-degraded".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "result diff candidate row is degraded".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "result_diff_profile_coverage_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "result_diff_encrypted_result_findings=1"));
}

#[test]
fn emits_code_diff_workflow_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G188".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "code bundle diff reveals missing lock metadata".to_string(),
        recommendation: "bind tool and image lock before code freeze publication".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0501".to_string(),
        severity: "critical".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        recommendation: "bind tool and image lock before code freeze publication".to_string(),
        affected_rows: vec!["h13".to_string()],
        source_appraisers: vec!["code-freeze".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "code_diff_trim_summary_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "code_diff_code_freeze_queue_entries=1"));
}

#[test]
fn emits_ambiguous_review_queue_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G192".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-performance".to_string(),
        row_id: "h5".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "ambiguous warning routed for manual review".to_string(),
        recommendation: "increase repetitions to at least 2".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0901".to_string(),
        severity: "warning".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        recommendation: "increase repetitions to at least 2".to_string(),
        affected_rows: vec!["h5".to_string()],
        source_appraisers: vec!["runtime-performance".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "ambiguous_review_warning_findings=1"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "ambiguous_review_queue_warning_entries=1"));
}

#[test]
fn emits_upstream_tool_issue_bundle_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G193".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "artifact-validity".to_string(),
        row_id: "h5".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "missing-tool-binding".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "suspected upstream tool issue due to tool binding break".to_string(),
        recommendation: "bind stage to at least one governed tool".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-1001".to_string(),
        severity: "critical".to_string(),
        failure_class: "missing-tool-binding".to_string(),
        recommendation: "bind stage to at least one governed tool".to_string(),
        affected_rows: vec!["h5".to_string()],
        source_appraisers: vec!["artifact-validity".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "upstream_tool_bundle_trim_align_call_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "upstream_tool_bundle_tool_binding_findings=1"));
}

#[test]
fn emits_data_issue_bundle_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G194".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "corpus-suitability".to_string(),
        row_id: "h10".to_string(),
        severity: "warning".to_string(),
        confidence: "high".to_string(),
        failure_class: "corpus-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "suspected data issue from corpus mismatch".to_string(),
        recommendation: "materialize corpus profile matching stage scientific claim".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-1101".to_string(),
        severity: "warning".to_string(),
        failure_class: "corpus-mismatch".to_string(),
        recommendation: "materialize corpus profile matching stage scientific claim".to_string(),
        affected_rows: vec!["h10".to_string()],
        source_appraisers: vec!["corpus-suitability".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "data_issue_bundle_validate_contam_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "data_issue_bundle_corpus_findings=1"));
}

#[test]
fn emits_runtime_issue_bundle_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G195".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-performance".to_string(),
        row_id: "h6".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "suspected runtime issue from under-sampled row".to_string(),
        recommendation: "increase repetitions to at least 2".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-1201".to_string(),
        severity: "warning".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        recommendation: "increase repetitions to at least 2".to_string(),
        affected_rows: vec!["h6".to_string()],
        source_appraisers: vec!["runtime-performance".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "runtime_issue_bundle_profile_summary_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "runtime_issue_bundle_runtime_findings=1"));
}

#[test]
fn emits_appraiser_confidence_score_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G197".to_string()];
    let findings = vec![
        AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h1".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "strong confidence finding".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        },
        AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "h4".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "weak confidence finding".to_string(),
            recommendation: "increase repetitions to at least 2".to_string(),
        },
    ];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "confidence_scores_high_confidence_findings=1"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "confidence_scores_manual_review_proxy=true"));
}

#[test]
fn emits_benchmark_history_analytics_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G198".to_string()];
    let findings = vec![
        AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h12".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "history trend: degraded imputation row".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        },
        AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h12".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "history trend: code drift in imputation branch".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
        },
    ];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "history_analytics_profile_coverage_impute_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "history_analytics_unique_failure_classes=2"));
}

#[test]
fn emits_stage_risk_ranking_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G199".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "stage risk ranking anchor from refuse row".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "stage_risk_ranking_trim_coverage_filter_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "stage_risk_ranking_refuse_rows=1"));
}
