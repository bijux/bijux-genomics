//! Owner: original hardening security regression scenarios.
use super::*;

#[test]
fn emits_encrypted_sharing_package_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G261".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "sharing package import indicates missing code bundle fields".to_string(),
        recommendation: "include required code sidecars for encrypted sharing packages".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "encrypted_sharing_profile_summary_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "encrypted_sharing_code_and_results_findings=1"));
}

#[test]
fn emits_reviewer_decrypt_profile_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G262".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "reviewer decrypt profile import indicates incomplete decrypt sidecars"
            .to_string(),
        recommendation: "scope decrypt profile to authorized bundles with complete sidecars"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "reviewer_decrypt_validate_validate_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "reviewer_decrypt_encrypted_findings=1"));
}

#[test]
fn emits_confidential_issue_bundle_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G263".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "confidential issue bundle import includes refused filter row evidence"
            .to_string(),
        recommendation: "minimize and encrypt only relevant artifacts in issue bundle output"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "confidential_bundle_trim_contamination_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "confidential_bundle_encrypted_findings=1"));
}

#[test]
fn emits_source_blame_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G264".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "database-compatibility".to_string(),
        row_id: "h5".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "database-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "source blame import points to database drift at align boundary".to_string(),
        recommendation: "attribute likely source changes from code/database/corpus locks"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "source_blame_profile_align_postprocess_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "source_blame_drift_findings=1"));
}

#[test]
fn emits_performance_regression_gate_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G265".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-performance".to_string(),
        row_id: "h7".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "performance gate import reports runtime baseline drift at coverage stage"
            .to_string(),
        recommendation: "gate against accepted runtime and IO baseline envelopes".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "performance_regression_trim_coverage_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "performance_regression_runtime_findings=1"));
}

#[test]
fn emits_scientific_regression_gate_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G266".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "scientific-invalidity".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "scientific gate import reports changed variant/caveat outcome at filter stage"
            .to_string(),
        recommendation: "gate against accepted scientific baseline outputs".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "scientific_regression_summary_filter_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "scientific_regression_scientific_findings=1"));
}

#[test]
fn emits_security_regression_gate_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G267".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "security gate import reports code-bundle encryption regression".to_string(),
        recommendation: "gate on encryption health and redact failures before promotion"
            .to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0267".to_string(),
        severity: "critical".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        recommendation: "gate on encryption health and redact failures before promotion"
            .to_string(),
        affected_rows: vec!["h13".to_string()],
        source_appraisers: vec!["code-freeze".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "security_regression_contamination_impute_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "security_regression_critical_queue_entries=1"));
}

#[test]
fn emits_replay_regression_gate_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G268".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "replay gate import reports missing reconstructability fields".to_string(),
        recommendation: "gate on complete replay fields in .code/.results bundles".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0268".to_string(),
        severity: "critical".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        recommendation: "gate on complete replay fields in .code/.results bundles".to_string(),
        affected_rows: vec!["h13".to_string()],
        source_appraisers: vec!["code-freeze".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "replay_regression_profile_summary_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "replay_regression_code_freeze_queue_entries=1"));
}

#[test]
fn emits_encryption_budget_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G274".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "encryption budget import reports sidecar/encryption health regression"
            .to_string(),
        recommendation: "pause when encryption or sidecar health drops below threshold".to_string(),
    }];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-0274".to_string(),
        severity: "critical".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        recommendation: "pause when encryption or sidecar health drops below threshold".to_string(),
        affected_rows: vec!["h13".to_string()],
        source_appraisers: vec!["code-freeze".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "encryption_budget_contamination_impute_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "encryption_budget_critical_queue_entries=1"));
}

#[test]
fn emits_reencryption_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G278".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "re-encryption import reports archival rewrap readiness gaps".to_string(),
        recommendation: "re-encrypt accepted bundles with key-rotation compatible metadata"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "reencryption_validate_validate_postprocess_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "reencryption_gate_triggered=true"));
}
