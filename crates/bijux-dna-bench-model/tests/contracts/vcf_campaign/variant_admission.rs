//! Owner: VCF campaign variant admission contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_validation_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G141".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "v2".to_string(),
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
        .any(|check| check.starts_with("validation_stage_count=2")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("validation_call_stage_bound=true")));
}

#[test]
fn emits_stats_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G142".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "v3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "stats-drift".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].goal_checks.iter().any(|check| check.starts_with("stats_degraded_rows=0")));
    assert!(entries[0].goal_checks.iter().any(|check| check.starts_with("stats_stage_bound=true")));
}

#[test]
fn emits_filtering_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G143".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].goal_checks.iter().any(|check| check.starts_with("filter_degraded_rows=1")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("filter_stage_bound=true")));
}

#[test]
fn emits_normalization_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G144".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "artifact-validity".to_string(),
        row_id: "v4".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "normalization-diff".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("normalization_repetition_floor=5")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("normalization_postprocess_stage_bound=true")));
}

#[test]
fn emits_reference_context_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G145".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "v5".to_string(),
        severity: "warning".to_string(),
        confidence: "high".to_string(),
        failure_class: "reference-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("reference_context_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("reference_context_panel_bound=true")));
}
