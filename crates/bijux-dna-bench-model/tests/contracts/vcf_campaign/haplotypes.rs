//! Owner: VCF campaign haplotypes contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_phasing_workflow_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G150".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0002".to_string(),
        severity: "warning".to_string(),
        failure_class: "panel-map-gap".to_string(),
        recommendation: "pin map/panel revisions".to_string(),
        affected_rows: vec!["v11".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].goal_checks.iter().any(|check| check.starts_with("phasing_stage_count=2")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("phasing_stage_bound=true")));
}

#[test]
fn emits_imputation_workflow_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G151".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("imputation_stage_count=4")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("imputation_impute_stage_bound=true")));
}
