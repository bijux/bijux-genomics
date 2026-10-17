//! Owner: VCF campaign cohort analysis contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_cohort_qc_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G152".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("cohort_qc_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("cohort_qc_qc_stage_bound=true")));
}

#[test]
fn emits_population_guardrail_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G153".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("population_guardrail_stage_count=4")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("population_guardrail_pca_stage_bound=true")));
}

#[test]
fn emits_roh_ibd_boundary_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G154".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].goal_checks.iter().any(|check| check.starts_with("roh_ibd_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("roh_ibd_roh_stage_bound=true")));
}

#[test]
fn emits_demography_boundary_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G155".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "v20".to_string(),
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
        .any(|check| check.starts_with("demography_refuse_rows=1")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("demography_stage_bound=true")));
}

#[test]
fn emits_population_handoff_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G158".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0003".to_string(),
        severity: "warning".to_string(),
        failure_class: "handoff-caveat".to_string(),
        recommendation: "tighten trust-class thresholds".to_string(),
        affected_rows: vec!["v20".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("population_handoff_stage_count=5")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("population_handoff_demography_bound=true")));
}
