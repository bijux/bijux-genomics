//! Owner: FASTQ campaign validation contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_validation_specific_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G101".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "r1".to_string(),
        severity: "warning".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-degraded".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0001".to_string(),
        severity: "warning".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        recommendation: "increase repetitions".to_string(),
        affected_rows: vec!["r1".to_string()],
        source_appraisers: vec!["runtime-performance".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("validation_rows_present=")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("validation_queue_entries=")));
}
