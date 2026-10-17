//! Owner: VCF campaign evidence contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_large_file_behavior_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G159".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-performance".to_string(),
        row_id: "v12".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("large_file_stage_count=4")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("large_file_impute_bound=true")));
}

#[test]
fn emits_full_template_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G160".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0004".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "resolve demography prerequisites".to_string(),
        affected_rows: vec!["v20".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("full_template_stage_count=10")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("full_template_core_stages_covered=10")));
}
