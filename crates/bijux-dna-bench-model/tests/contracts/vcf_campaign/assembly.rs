//! Owner: VCF campaign assembly contracts.
#![cfg(test)]
use super::*;

#[test]
fn goal_entries_map_rows_findings_and_queue() {
    let matrix = matrix_fixture();
    let selected = vec!["G141".to_string(), "G147".to_string(), "G149".to_string()];
    let findings = vec![
        AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "v2".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        },
        AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "v7".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        },
    ];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0001".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        affected_rows: vec!["v7".to_string(), "v1".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];

    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].goal_id, "G141");
    assert_eq!(entries[0].hardening_entries[0].affected_rows, ["v1"]);
    assert_eq!(entries[1].goal_id, "G147");
    assert_eq!(entries[1].hardening_entries[0].affected_rows, ["v7"]);
    assert_eq!(entries[1].status, "requires-hardening");
    assert_eq!(entries[2].goal_id, "G149");
    assert_eq!(entries[2].matched_rows.len(), 2);

    let summary = summarize(&entries);
    assert_eq!(summary.total_goals, 3);
    assert_eq!(summary.total_rows, 5);
    assert_eq!(summary.status_counts.get("requires-hardening"), Some(&2));
}

#[test]
fn summary_counts_mixed_status_entries() {
    let entries = vec![
        super::VcfBenchmarkGoalEntry {
            goal_id: "G141".to_string(),
            title: "benchmark VCF validation".to_string(),
            stage_ids: vec!["vcf.call".to_string(), "vcf.filter".to_string()],
            scenarios: Vec::new(),
            matched_rows: vec![VcfMatrixRowRef {
                row_id: "v1".to_string(),
                stage_id: "vcf.call".to_string(),
                tool_id: "bcftools_v1".to_string(),
                readiness_class: "ready".to_string(),
                repetitions: 5,
            }],
            appraisal_findings: Vec::new(),
            hardening_entries: Vec::new(),
            goal_checks: Vec::new(),
            status: "ready-for-benchmark-run".to_string(),
        },
        super::VcfBenchmarkGoalEntry {
            goal_id: "G142".to_string(),
            title: "benchmark VCF stats".to_string(),
            stage_ids: vec!["vcf.stats".to_string()],
            scenarios: Vec::new(),
            matched_rows: Vec::new(),
            appraisal_findings: Vec::new(),
            hardening_entries: Vec::new(),
            goal_checks: Vec::new(),
            status: "missing-stage-binding".to_string(),
        },
    ];
    let summary = summarize(&entries);
    assert_eq!(summary.total_rows, 1);
    assert_eq!(summary.total_goals, 2);
    assert_eq!(summary.status_counts.get("ready-for-benchmark-run"), Some(&1));
    assert_eq!(summary.status_counts.get("missing-stage-binding"), Some(&1));
}
