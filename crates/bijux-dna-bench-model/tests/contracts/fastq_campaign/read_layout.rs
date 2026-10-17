//! Owner: FASTQ campaign read layout contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_read_name_specific_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G102".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "r2".to_string(),
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
        .any(|check| check.starts_with("normalization_stage_bound=true")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("normalization_refuse_rows=")));
}

#[test]
fn emits_pair_repair_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G103".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-performance".to_string(),
        row_id: "r3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("pair_repair_rows_present=")));
}

#[test]
fn emits_interleave_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G104".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("interleave_stage_pair_covered=")));
}

#[test]
fn emits_lane_concat_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G105".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("lane_concat_stage_bound=")));
}

#[test]
fn emits_demultiplex_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G106".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].goal_checks.iter().any(|check| check.starts_with("demux_stage_bound=")));
}
