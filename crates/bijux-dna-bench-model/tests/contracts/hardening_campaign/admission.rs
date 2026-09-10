//! Owner: original hardening admission regression scenarios.
use super::*;

#[test]
fn hardening_catalog_covers_declared_id_span() {
    assert_eq!(HARDENING_GOALS_CATALOG.len(), 110);
    assert_eq!(HARDENING_GOALS_CATALOG[0].goal_id, "G171");
    assert_eq!(HARDENING_GOALS_CATALOG[109].goal_id, "G280");
}

#[test]
fn selected_goal_ids_reject_unknown_entries() {
    let err = selected_goal_ids(Some("G171,G999")).expect_err("expected invalid goal");
    assert!(err.to_string().contains("unknown hardening goals requested"));
}

#[test]
fn goal_entries_map_rows_findings_and_queue() {
    let matrix = matrix_fixture();
    let selected = vec!["G171".to_string(), "G175".to_string(), "G179".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
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
        affected_rows: vec!["h10".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];

    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].goal_id, "G171");
    assert_eq!(entries[1].goal_id, "G175");
    assert_eq!(entries[2].goal_id, "G179");

    let summary = summarize(&entries);
    assert_eq!(summary.total_goals, 3);
    assert_eq!(summary.status_counts.get("requires-hardening"), Some(&1));
}

#[test]
fn summary_counts_mixed_status_entries() {
    let entries = vec![
        super::HardeningBenchmarkGoalEntry {
            goal_id: "G171".to_string(),
            title: "add per-stage hardening playbooks".to_string(),
            stage_ids: vec!["fastq.validate_reads".to_string()],
            scenarios: Vec::new(),
            matched_rows: vec![HardeningMatrixRowRef {
                row_id: "h1".to_string(),
                stage_id: "fastq.validate_reads".to_string(),
                tool_id: "tool_v1".to_string(),
                readiness_class: "ready".to_string(),
                repetitions: 3,
            }],
            appraisal_findings: Vec::new(),
            hardening_entries: Vec::new(),
            goal_checks: Vec::new(),
            status: "ready-for-benchmark-run".to_string(),
        },
        super::HardeningBenchmarkGoalEntry {
            goal_id: "G172".to_string(),
            title: "add per-tool hardening playbooks".to_string(),
            stage_ids: vec!["fastq.trim_reads".to_string()],
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
