//! Owner: BAM campaign contamination behavior.
use super::*;

#[test]
fn emits_mito_contamination_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G133".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "b12".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "readiness-degraded".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("mito_contam_stage_bound=true")));
}

#[test]
fn emits_nuclear_contamination_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G134".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "b12".to_string(),
        severity: "warning".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-degraded".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("nuclear_contam_stage_bound=true")));
}
