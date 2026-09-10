//! Owner: original hardening operations regression scenarios.
use super::*;

#[test]
fn emits_nightly_campaign_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G269".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "nightly campaign import captured refusal drift in filter stage".to_string(),
        recommendation: "rotate nightly subset and track drift history for refuses".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "nightly_campaign_validate_align_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "nightly_campaign_findings_count=1"));
}

#[test]
fn emits_monthly_campaign_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G270".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h7".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "monthly full-campaign import reports coverage runtime drift".to_string(),
        recommendation: "compare monthly full-campaign outcomes against accepted baselines"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "monthly_campaign_profile_coverage_call_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "monthly_campaign_findings_count=1"));
}

#[test]
fn emits_cost_pruning_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G271".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "cost-aware pruning import marks refusal row as high-value keep candidate"
            .to_string(),
        recommendation: "prune low-value rows while preserving refusal and high-risk evidence rows"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "cost_pruning_profile_coverage_filter_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "cost_pruning_signal_count=1"));
}

#[test]
fn emits_failure_budget_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G272".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "failure budget import exceeds refusal threshold in filter stage".to_string(),
        recommendation: "pause campaign when failure budget threshold is exceeded".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "failure_budget_validate_align_filter_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "failure_budget_triggered=true"));
}

#[test]
fn emits_storage_budget_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G273".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h6".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "storage budget import reports shared-storage runtime drift".to_string(),
        recommendation: "pause before storage exhaustion and rebalance archive/staging policy"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "storage_budget_profile_summary_postprocess_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "storage_budget_triggered=true"));
}

#[test]
fn emits_appraiser_budget_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G275".to_string()];
    let findings = vec![
        AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h11".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "appraiser health import reports degraded stats interpretation".to_string(),
            recommendation: "monitor appraiser failure rate and pause if interpretation drops"
                .to_string(),
        },
        AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h11".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "scientific-invalidity".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "appraiser health import reports critical stats interpretation failure"
                .to_string(),
            recommendation: "pause campaign when appraiser health budget exceeds threshold"
                .to_string(),
        },
    ];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "appraiser_budget_validate_validate_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "appraiser_budget_triggered=true"));
}

#[test]
fn emits_retention_tier_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G276".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "retention tier import marks code bundles for full encrypted retention"
            .to_string(),
        recommendation:
            "retain high-value encrypted bundles and purge low-value intermediates safely"
                .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "retention_tier_profile_summary_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "retention_tier_encrypted_findings=1"));
}

#[test]
fn emits_local_archive_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G277".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "local archive import reports missing investigation metadata fields".to_string(),
        recommendation: "archive decrypted investigations in a structured local private format"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "local_archive_profile_coverage_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "local_archive_encrypted_findings=1"));
}
