//! Owner: BAM campaign resources behavior.
use super::*;

#[test]
fn emits_resource_estimation_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G139".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-performance".to_string(),
        row_id: "b8".to_string(),
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
        .any(|check| check.starts_with("resource_qc_pre_bound=true")));
}

#[test]
fn emits_full_bam_template_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G140".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("full_template_core_stages_covered=")));
}
