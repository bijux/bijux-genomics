//! Owner: FASTQ campaign assembly contracts.
#![cfg(test)]
use super::*;

#[test]
fn goal_entries_map_rows_findings_and_queue() {
    let matrix = matrix_fixture();
    let selected = vec!["G101".to_string(), "G102".to_string(), "G109".to_string()];
    let findings = vec![
        AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "r1".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        },
        AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "r2".to_string(),
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
        affected_rows: vec!["r2".to_string(), "r3".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];

    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].goal_id, "G101");
    assert_eq!(entries[0].matched_rows.len(), 1);
    assert_eq!(entries[1].goal_id, "G102");
    assert_eq!(entries[1].status, "requires-hardening");
    assert_eq!(entries[1].hardening_entries[0].affected_rows, ["r2"]);
    assert_eq!(entries[2].goal_id, "G109");
    assert_eq!(entries[2].matched_rows.len(), 1);

    let summary = summarize(&entries);
    assert_eq!(summary.total_goals, 3);
    assert!(summary.status_counts.contains_key("requires-hardening"));
}

#[test]
fn summarize_counts_statuses() {
    let entries = vec![
        super::FastqBenchmarkGoalEntry {
            goal_id: "G101".to_string(),
            title: "benchmark FASTQ validation".to_string(),
            stage_ids: vec!["fastq.validate_reads".to_string()],
            scenarios: Vec::new(),
            matched_rows: vec![FastqMatrixRowRef {
                row_id: "r1".to_string(),
                stage_id: "fastq.validate_reads".to_string(),
                tool_id: "seqkit_v2".to_string(),
                readiness_class: "ready".to_string(),
                repetitions: 3,
            }],
            appraisal_findings: Vec::new(),
            hardening_entries: Vec::new(),
            goal_checks: Vec::new(),
            status: "ready-for-benchmark-run".to_string(),
        },
        super::FastqBenchmarkGoalEntry {
            goal_id: "G102".to_string(),
            title: "benchmark FASTQ read-name normalization".to_string(),
            stage_ids: vec!["fastq.normalize_read_names".to_string()],
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
