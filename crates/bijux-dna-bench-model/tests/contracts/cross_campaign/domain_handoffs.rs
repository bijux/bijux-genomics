//! Owner: Cross-domain campaign domain handoffs contracts.
#![cfg(test)]
use super::*;

#[test]
fn emits_modern_cross_workflow_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G161".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("modern_workflow_stage_count=4")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("modern_workflow_summary_to_stats_bound=true")));
}

#[test]
fn emits_adna_cross_workflow_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G162".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("adna_workflow_stage_count=2")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("adna_workflow_damage_bound=true")));
}

#[test]
fn emits_bam_to_vcf_handoff_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G163".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("bam_to_vcf_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("bam_to_vcf_call_bound=true")));
}

#[test]
fn emits_fastq_to_vcf_chain_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G164".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("fastq_to_vcf_stage_count=3")));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check.starts_with("fastq_to_vcf_gl_bound=true")));
}
