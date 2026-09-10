//! Owner: original hardening execution regression scenarios.
use super::*;

#[test]
fn emits_image_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G175".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "artifact-validity".to_string(),
        row_id: "h6".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "image-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "image mismatch".to_string(),
        recommendation: "prepare or stage matching image before benchmark".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0003".to_string(),
        severity: "critical".to_string(),
        failure_class: "image-mismatch".to_string(),
        recommendation: "prepare or stage matching image before benchmark".to_string(),
        affected_rows: vec!["h6".to_string()],
        source_appraisers: vec!["artifact-validity".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "image_hardening_mapping_summary_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "image_hardening_image_mismatch_findings=1"));
}

#[test]
fn emits_slurm_wrapper_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G176".to_string()];
    let findings = vec![
        AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "h7".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "benchmark row has too few repetitions".to_string(),
            recommendation: "increase repetitions to at least 2".to_string(),
        },
        AppraisalFinding {
            appraiser_id: "reproducibility".to_string(),
            row_id: "h6".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "reproducibility-low-repeats".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "low repeats for reproducibility".to_string(),
            recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
        },
    ];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "wrapper_hardening_runtime_findings=1"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "wrapper_hardening_repro_low_repeat_findings=1"));
}

#[test]
fn emits_resource_tuning_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G177".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-performance".to_string(),
        row_id: "h7".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "benchmark row has too few repetitions".to_string(),
        recommendation: "increase repetitions to at least 2".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "resource_hardening_degraded_rows=2"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "resource_hardening_runtime_under_sampled_findings=1"));
}

#[test]
fn emits_io_staging_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G178".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h11".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "code freeze missing lock metadata".to_string(),
        recommendation: "bind tool and image lock before code freeze publication".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "io_hardening_profile_to_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "io_hardening_code_freeze_findings=1"));
}

#[test]
fn emits_encryption_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G179".to_string()];
    let findings = vec![
        AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h13".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "code freeze metadata incomplete".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
        },
        AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h12".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "readiness degraded".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        },
    ];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0004".to_string(),
        severity: "critical".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        recommendation: "bind tool and image lock before code freeze publication".to_string(),
        affected_rows: vec!["h13".to_string()],
        source_appraisers: vec!["code-freeze".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "security_hardening_encrypted_scope_findings=2"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "security_hardening_critical_queue_entries=1"));
}

#[test]
fn emits_replay_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G180".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "replay snapshot missing lock metadata".to_string(),
        recommendation: "bind tool and image lock before code freeze publication".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0005".to_string(),
        severity: "critical".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        recommendation: "bind tool and image lock before code freeze publication".to_string(),
        affected_rows: vec!["h13".to_string()],
        source_appraisers: vec!["code-freeze".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "replay_hardening_chain_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "replay_hardening_code_freeze_queue_entries=1"));
}

#[test]
fn emits_nondeterminism_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G181".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "reproducibility".to_string(),
        row_id: "h6".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "reproducibility-low-repeats".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "low repeats for reproducibility".to_string(),
        recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "nondeterminism_profile_summary_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "nondeterminism_repro_runtime_findings=1"));
}

#[test]
fn emits_output_schema_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G182".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "schema bundle missing lock metadata".to_string(),
        recommendation: "bind tool and image lock before code freeze publication".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "output_schema_validate_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "output_schema_code_freeze_findings=1"));
}

#[test]
fn emits_accepted_baseline_workflow_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G189".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "reproducibility".to_string(),
        row_id: "h1".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "reproducibility-low-repeats".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "baseline candidate requires repeat strengthening".to_string(),
        recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0601".to_string(),
        severity: "warning".to_string(),
        failure_class: "reproducibility-low-repeats".to_string(),
        recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
        affected_rows: vec!["h1".to_string()],
        source_appraisers: vec!["reproducibility".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "baseline_workflow_validate_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "baseline_workflow_critical_findings=0"));
}

#[test]
fn emits_regression_workflow_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G190".to_string()];
    let findings = vec![
        AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "regression rerun surfaced refuse row".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        },
        AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h13".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "regression rerun surfaced replay/code drift".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
        },
    ];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0701".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        affected_rows: vec!["h10".to_string(), "h13".to_string()],
        source_appraisers: vec!["failure-class".to_string(), "code-freeze".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "regression_workflow_profile_coverage_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "regression_workflow_critical_findings=1"));
}

#[test]
fn emits_hardening_sprint_generator_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G191".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "high-risk row selected for sprint".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0801".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        affected_rows: vec!["h10".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "sprint_generator_high_risk_findings=1"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "sprint_generator_non_bookkeeping_entries=1"));
}

#[test]
fn emits_campaign_salvage_workflow_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G196".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-performance".to_string(),
        row_id: "h7".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "partial campaign row preserved for salvage".to_string(),
        recommendation: "increase repetitions to at least 2".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].goal_checks.iter().any(|check| check == "campaign_salvage_partial_rows=3"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "campaign_salvage_noncritical_findings=1"));
}

#[test]
fn emits_hardening_acceptance_rerun_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G200".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-1301".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        affected_rows: vec!["h10".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "acceptance_rerun_validate_align_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "acceptance_rerun_before_after_queue_entries=1"));
}
