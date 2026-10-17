//! Owner: Cross-domain campaign reviewer evidence contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_reviewer_bundle_generation_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G170".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0004".to_string(),
        severity: "warning".to_string(),
        failure_class: "bundle-import-gap".to_string(),
        recommendation: "verify decrypt/import pipeline".to_string(),
        affected_rows: vec!["c10".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("reviewer_bundle_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("reviewer_bundle_call_bound=true")));
}
