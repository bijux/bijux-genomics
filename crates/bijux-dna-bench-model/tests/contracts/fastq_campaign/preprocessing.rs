//! Owner: FASTQ campaign preprocessing contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_adapter_detection_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G108".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "backend-equivalence".to_string(),
        row_id: "r3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "single-backend".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("adapter_equivalence_findings=")));
}

#[test]
fn emits_trimming_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G109".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "r3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "readiness-degraded".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].goal_checks.iter().any(|check| check.starts_with("trim_backend_findings=")));
}

#[test]
fn emits_umi_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G110".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "r4".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("umi_code_freeze_findings=")));
}

#[test]
fn emits_duplicate_complexity_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G111".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "r5".to_string(),
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
        .any(|check| check.starts_with("complexity_stage_bound=")));
}

#[test]
fn emits_edna_preprocessing_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G116".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "r10".to_string(),
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
        .any(|check| check.starts_with("edna_primer_stage_bound=true")));
}
