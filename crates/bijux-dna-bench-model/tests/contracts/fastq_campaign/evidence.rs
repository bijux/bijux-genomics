//! Owner: FASTQ campaign evidence contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_chunked_processing_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G117".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "backend-equivalence".to_string(),
        row_id: "r11".to_string(),
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
        .any(|check| check.starts_with("chunk_profile_stage_bound=true")));
}

#[test]
fn emits_qc_manifest_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G118".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "reproducibility".to_string(),
        row_id: "r12".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "reproducibility-low-repeats".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("qc_manifest_stage_bound=true")));
}

#[test]
fn emits_provenance_capture_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G119".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "r13".to_string(),
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
        .any(|check| check.starts_with("provenance_stage_bound=true")));
}

#[test]
fn emits_full_template_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G120".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("template_core_stages_covered=")));
}
