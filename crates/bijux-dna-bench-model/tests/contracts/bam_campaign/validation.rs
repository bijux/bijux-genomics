//! Owner: BAM campaign validation behavior.
use super::*;

#[test]
fn emits_validation_boundary_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G121".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "b9".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("validation_stage_bound=true")));
}

#[test]
fn emits_sort_index_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G124".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("sort_index_validate_stage_bound=true")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("sort_index_mapping_summary_bound=true")));
}

#[test]
fn emits_read_group_propagation_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G125".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "b1".to_string(),
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
        .any(|check| check.starts_with("read_group_validate_stage_bound=true")));
}

#[test]
fn emits_bam_merge_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G126".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "b3".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("bam_merge_qc_pre_stage_bound=true")));
}
