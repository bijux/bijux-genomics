//! Owner: Cross-domain campaign execution policy contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_batch_fanout_fanin_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G165".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("batch_fan_stage_count=4")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("batch_fan_summary_stats_bound=true")));
}

#[test]
fn emits_partial_failure_policy_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G166".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("partial_failure_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("partial_failure_vcf_bound=true")));
}
