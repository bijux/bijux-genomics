//! Owner: FASTQ campaign depletion contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_host_depletion_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G112".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "r6".to_string(),
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
        .any(|check| check.starts_with("host_depletion_stage_bound=true")));
}

#[test]
fn emits_rrna_depletion_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G113".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("rrna_depletion_stage_bound=true")));
}

#[test]
fn emits_contaminant_depletion_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G114".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "r8".to_string(),
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
        .any(|check| check.starts_with("contaminant_stage_bound=true")));
}

#[test]
fn emits_taxonomy_screening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G115".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "r9".to_string(),
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
        .any(|check| check.starts_with("taxonomy_stage_bound=true")));
}
