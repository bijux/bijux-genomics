//! Owner: VCF campaign calling contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_damage_aware_filtering_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G146".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "v6".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "damage-policy-drift".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("damage_filter_repetition_floor=5")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("damage_filter_stage_bound=true")));
}

#[test]
fn emits_diploid_boundary_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G147".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "v7".to_string(),
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
        .any(|check| check.starts_with("diploid_queue_entries=0")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("diploid_stage_bound=true")));
}

#[test]
fn emits_pseudohaploid_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G148".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "v8".to_string(),
        severity: "warning".to_string(),
        confidence: "high".to_string(),
        failure_class: "uncertainty-caveat".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("pseudohaploid_repetition_floor=5")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("pseudohaploid_stage_bound=true")));
}

#[test]
fn emits_gl_workflow_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G149".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "v10".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "gl-propagation-drift".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].goal_checks.iter().any(|check| check.starts_with("gl_stage_count=2")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("gl_propagation_stage_bound=true")));
}
