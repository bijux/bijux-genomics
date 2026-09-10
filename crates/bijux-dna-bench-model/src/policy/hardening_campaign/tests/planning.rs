//! Owner: original hardening planning regression scenarios.
use super::*;

#[test]
fn emits_campaign_coverage_planner_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G228".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h7".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "coverage planner includes runtime outlier row".to_string(),
        recommendation: "rebalance coverage planner row selection by runtime budget".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "campaign_coverage_profile_coverage_impute_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "campaign_coverage_findings_count=1"));
}

#[test]
fn emits_budget_planner_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G229".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h7".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "budget planner detects expensive coverage-stage profile".to_string(),
        recommendation: "rebalance budget planner row-set for high-cost boundaries".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "budget_planner_align_coverage_postprocess_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "budget_planner_findings_count=1"));
}

#[test]
fn emits_campaign_split_quota_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G230".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-1901".to_string(),
        severity: "warning".to_string(),
        failure_class: "runtime-outlier".to_string(),
        recommendation: "split campaign rows by quota windows".to_string(),
        affected_rows: vec!["h6".to_string()],
        source_appraisers: vec!["runtime-profile".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "campaign_split_validate_summary_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "campaign_split_queue_entries=1"));
}

#[test]
fn emits_campaign_lock_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G231".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "campaign lock misses postprocess provenance anchor".to_string(),
        recommendation: "freeze postprocess provenance and lock digest state".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "campaign_lock_validate_validate_postprocess_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "campaign_lock_findings_count=1"));
}

#[test]
fn emits_campaign_rerun_lock_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G232".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "reproducibility".to_string(),
        row_id: "h3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "reproducibility-low-repeats".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "rerun-from-lock profile stage needs repeat reinforcement".to_string(),
        recommendation: "set rerun repetitions >= 3 for lock replay confidence".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "campaign_rerun_profile_summary_postprocess_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "campaign_rerun_findings_count=1"));
}

#[test]
fn emits_hardening_queue_subset_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G233".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-2001".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "rerun only hardening queue rows for validation".to_string(),
        affected_rows: vec!["h10".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "queue_subset_validate_align_filter_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "queue_subset_queue_entries=1"));
}

#[test]
fn emits_campaign_priority_label_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G234".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h7".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "priority labels need cost-value rebalance for coverage row".to_string(),
        recommendation: "raise priority for high scientific value low-cost rows".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "priority_label_profile_coverage_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "priority_label_findings_count=1"));
}

#[test]
fn emits_warm_start_campaign_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G235".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h6".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "readiness-degraded".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "warm-start campaign has degraded pre-locked align surface".to_string(),
        recommendation: "restore ready warm-start assets before submit".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "warm_start_validate_summary_call_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "warm_start_findings_count=1"));
}

#[test]
fn emits_all_in_one_preflight_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G236".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-2101".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "resolve preflight refusal row before campaign submit".to_string(),
        affected_rows: vec!["h10".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "all_in_one_preflight_validate_validate_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "all_in_one_preflight_queue_entries=1"));
}

#[test]
fn emits_all_in_one_import_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G237".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "all-in-one import profile stage has runtime caveat".to_string(),
        recommendation: "optimize import pipeline and re-index replay artifacts".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "all_in_one_import_profile_summary_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "all_in_one_import_findings_count=1"));
}

#[test]
fn emits_all_in_one_report_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G238".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "scientific-caveat".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "all-in-one report requires caveat link to postprocess drift".to_string(),
        recommendation: "include postprocess caveat links in generated summary reports".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "all_in_one_report_profile_coverage_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "all_in_one_report_findings_count=1"));
}

#[test]
fn emits_campaign_comparison_report_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G239".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "reproducibility".to_string(),
        row_id: "h5".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "reproducibility-low-repeats".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "campaign comparison detected unstable align boundary".to_string(),
        recommendation: "increase repetitions before campaign drift comparison".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "campaign_comparison_align_call_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "campaign_comparison_findings_count=1"));
}

#[test]
fn emits_benchmark_docs_hint_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G240".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "benchmark output indicates docs caveat for filter refusal behavior".to_string(),
        recommendation: "add docs hint when refusal behavior changes user-visible guidance"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "benchmark_docs_validate_filter_postprocess_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "benchmark_docs_findings_count=1"));
}
