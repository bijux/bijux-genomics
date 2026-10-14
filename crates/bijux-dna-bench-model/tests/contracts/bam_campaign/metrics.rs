//! Owner: BAM campaign metrics behavior.
use super::*;

#[test]
fn emits_mapping_summary_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G129".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "artifact-validity".to_string(),
        row_id: "b7".to_string(),
        severity: "warning".to_string(),
        confidence: "high".to_string(),
        failure_class: "image-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("mapping_summary_stage_bound=true")));
}

#[test]
fn emits_coverage_regime_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G130".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "b8".to_string(),
        severity: "warning".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-degraded".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("coverage_stage_bound=true")));
}
