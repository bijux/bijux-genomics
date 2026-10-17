//! Owner: FASTQ campaign sampling contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_subsample_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G107".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "reproducibility".to_string(),
        row_id: "r3".to_string(),
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
        .any(|check| check.starts_with("subsample_repro_findings=")));
}
