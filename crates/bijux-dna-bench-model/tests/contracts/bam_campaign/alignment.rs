//! Owner: BAM campaign alignment behavior.
use super::*;

#[test]
fn emits_bwa_alignment_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G122".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-performance".to_string(),
        row_id: "b2".to_string(),
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
        .any(|check| check.starts_with("bwa_alignment_stage_bound=true")));
}

#[test]
fn emits_bowtie2_alignment_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G123".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "backend-equivalence".to_string(),
        row_id: "b2".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "single-backend".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("bowtie2_alignment_stage_bound=true")));
}
