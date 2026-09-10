//! Owner: original hardening comparisons regression scenarios.
use super::*;

#[test]
fn emits_full_fastq_campaign_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G241".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "full FASTQ campaign has profile-stage runtime outlier".to_string(),
        recommendation: "tune FASTQ-stage runtime profile before full promotion".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "full_fastq_validate_trim_profile_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "full_fastq_findings_count=1"));
}

#[test]
fn emits_full_bam_campaign_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G242".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h5".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "full BAM campaign has align-stage runtime outlier".to_string(),
        recommendation: "tune BAM-stage runtime before full campaign promotion".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "full_bam_validate_align_summary_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "full_bam_findings_count=1"));
}

#[test]
fn emits_full_vcf_campaign_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G243".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "full VCF campaign includes filter refusal boundary".to_string(),
        recommendation: "resolve filter refusal before full VCF promotion".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "full_vcf_call_filter_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "full_vcf_findings_count=1"));
}

#[test]
fn emits_full_cross_domain_campaign_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G244".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "reproducibility".to_string(),
        row_id: "h5".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "reproducibility-low-repeats".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "cross-domain campaign align stage lacks repeat confidence".to_string(),
        recommendation: "increase repetitions for cross-domain campaign parity".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "full_cross_domain_profile_align_call_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "full_cross_domain_findings_count=1"));
}

#[test]
fn emits_all_domain_series_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G245".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "all-domain series lacks complete postprocess code-freeze fields".to_string(),
        recommendation: "freeze series provenance for all-domain campaign replay".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "all_domain_series_validate_validate_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "all_domain_series_findings_count=1"));
}

#[test]
fn emits_local_slurm_parity_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G246".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "reproducibility".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "local/slurm parity mismatch at filter refusal boundary".to_string(),
        recommendation: "align local and Slurm runtime flags for parity checks".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "local_slurm_parity_trim_coverage_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "local_slurm_parity_findings_count=1"));
}

#[test]
fn emits_image_version_drift_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G247".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h6".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "image-version comparison shows mapping-summary runtime drift".to_string(),
        recommendation: "pin image digest or tune image runtime profile".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "image_version_trim_summary_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "image_version_findings_count=1"));
}

#[test]
fn emits_database_version_drift_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G248".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "database-compatibility".to_string(),
        row_id: "h12".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "database-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "database-version comparison shows imputation boundary mismatch".to_string(),
        recommendation: "align database version rollout across campaign rows".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "database_version_align_call_impute_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "database_version_findings_count=1"));
}

#[test]
fn emits_corpus_scale_comparison_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G249".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "corpus-scale comparison exposes scale-only refusal path".to_string(),
        recommendation: "partition scale tiers and isolate refusal-only large-corpus rows"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "corpus_scale_profile_coverage_filter_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "corpus_scale_findings_count=1"));
}

#[test]
fn emits_storage_mode_comparison_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G250".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "cold/warm storage comparison shows profile-stage IO drift".to_string(),
        recommendation: "benchmark cache-state and staging policy across storage modes".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "storage_mode_profile_summary_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "storage_mode_findings_count=1"));
}
