//! Owner: BAM campaign assembly behavior.
use super::*;

#[test]
fn goal_entries_map_rows_findings_and_queue() {
    let matrix = matrix_fixture();
    let selected = vec!["G121".to_string(), "G127".to_string(), "G130".to_string()];
    let findings = vec![
        AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "b1".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        },
        AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "b8".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "coverage-uncertain".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        },
    ];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0001".to_string(),
        severity: "warning".to_string(),
        failure_class: "coverage-uncertain".to_string(),
        recommendation: "tighten coverage policy".to_string(),
        affected_rows: vec!["b8".to_string(), "b1".to_string()],
        source_appraisers: vec!["scientific-output".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].goal_id, "G121");
    assert_eq!(entries[1].goal_id, "G127");
    assert_eq!(entries[2].goal_id, "G130");
    assert_eq!(entries[0].hardening_entries[0].affected_rows, ["b1"]);
    assert_eq!(entries[2].hardening_entries[0].affected_rows, ["b8"]);

    let summary = summarize(&entries);
    assert_eq!(summary.total_goals, 3);
}

#[test]
fn summarize_counts_statuses() {
    let entries = vec![
        super::BamBenchmarkGoalEntry {
            goal_id: "G121".to_string(),
            title: "benchmark BAM/SAM/CRAM validation boundaries".to_string(),
            stage_ids: vec!["bam.validate".to_string()],
            scenarios: Vec::new(),
            matched_rows: vec![BamMatrixRowRef {
                row_id: "b1".to_string(),
                stage_id: "bam.validate".to_string(),
                tool_id: "samtools_v1".to_string(),
                readiness_class: "ready".to_string(),
                repetitions: 3,
            }],
            appraisal_findings: Vec::new(),
            hardening_entries: Vec::new(),
            goal_checks: Vec::new(),
            status: "ready-for-benchmark-run".to_string(),
        },
        super::BamBenchmarkGoalEntry {
            goal_id: "G122".to_string(),
            title: "benchmark BWA-style alignment".to_string(),
            stage_ids: vec!["bam.align".to_string()],
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
