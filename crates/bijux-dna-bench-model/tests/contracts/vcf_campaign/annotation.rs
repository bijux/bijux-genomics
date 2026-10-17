//! Owner: VCF campaign annotation contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_annotation_provenance_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G156".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "artifact-validity".to_string(),
        row_id: "v4".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "annotation-drift".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("annotation_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("annotation_postprocess_stage_bound=true")));
}

#[test]
fn emits_structural_variant_boundary_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G157".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("sv_boundary_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("sv_boundary_qc_stage_bound=true")));
}
