//! Owner: Cross-domain campaign assembly contracts.
#![cfg(test)]
use super::*;

#[test]
fn goal_entries_map_rows_findings_and_queue() {
    let matrix = matrix_fixture();
    let selected = vec!["G161".to_string(), "G166".to_string(), "G169".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "c10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "x".to_string(),
        recommendation: "y".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0001".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        affected_rows: vec!["c10".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];

    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].goal_id, "G161");
    assert_eq!(entries[1].goal_id, "G166");
    assert_eq!(entries[2].goal_id, "G169");

    let summary = summarize(&entries);
    assert_eq!(summary.total_goals, 3);
    assert_eq!(summary.status_counts.get("requires-hardening"), Some(&1));
}

#[test]
fn summary_counts_mixed_status_entries() {
    let entries = vec![
        super::CrossBenchmarkGoalEntry {
            goal_id: "G161".to_string(),
            title: "benchmark FASTQ-to-BAM modern workflow".to_string(),
            stage_ids: vec![BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN.to_string()],
            scenarios: Vec::new(),
            matched_rows: vec![CrossMatrixRowRef {
                row_id: "c1".to_string(),
                stage_id: BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN.to_string(),
                tool_id: "seqkit_v2=>bwa_v0".to_string(),
                readiness_class: "ready".to_string(),
                repetitions: 5,
            }],
            appraisal_findings: Vec::new(),
            hardening_entries: Vec::new(),
            goal_checks: Vec::new(),
            status: "ready-for-benchmark-run".to_string(),
        },
        super::CrossBenchmarkGoalEntry {
            goal_id: "G162".to_string(),
            title: "benchmark FASTQ-to-BAM aDNA workflow".to_string(),
            stage_ids: vec![BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE.to_string()],
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
