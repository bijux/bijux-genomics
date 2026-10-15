//! Owner: Cross-domain campaign source integrity contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_sample_sheet_validation_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G167".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("sample_sheet_stage_count=2")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("sample_sheet_validate_align_bound=true")));
}

#[test]
fn emits_reference_change_propagation_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G168".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("reference_change_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("reference_change_call_bound=true")));
}

#[test]
fn emits_contamination_propagation_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G169".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("contamination_flow_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("contamination_flow_bam_to_vcf_bound=true")));
}
