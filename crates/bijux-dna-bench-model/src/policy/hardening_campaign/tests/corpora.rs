//! Owner: original hardening corpora regression scenarios.
use super::*;

#[test]
fn emits_modern_wgs_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G201".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h5".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "readiness-degraded".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "modern WGS readiness needs tuning".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "modern_wgs_validate_align_call_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "modern_wgs_findings_count=1"));
}

#[test]
fn emits_adna_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G202".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "aDNA boundary triggers refuse contamination-ready path".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "adna_corpus_trim_contam_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "adna_corpus_contamination_findings=1"));
}

#[test]
fn emits_edna_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G203".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "corpus-suitability".to_string(),
        row_id: "h3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "corpus-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "eDNA corpus boundary warning".to_string(),
        recommendation: "materialize corpus profile matching stage scientific claim".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "edna_corpus_profile_contam_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "edna_corpus_warning_findings=1"));
}

#[test]
fn emits_lowpass_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G204".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "reproducibility".to_string(),
        row_id: "h12".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "reproducibility-low-repeats".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "low-pass imputation needs repeat reinforcement".to_string(),
        recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "lowpass_corpus_profile_impute_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "lowpass_corpus_findings_count=1"));
}

#[test]
fn emits_contamination_heavy_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G205".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-1401".to_string(),
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
        .any(|check| check == "contamination_heavy_validate_contam_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "contamination_heavy_queue_entries=1"));
}

#[test]
fn emits_nonhuman_reference_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G206".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "non-human reference bundle needs lock metadata".to_string(),
        recommendation: "bind tool and image lock before code freeze publication".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "nonhuman_corpus_validate_align_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "nonhuman_corpus_code_scope_findings=1"));
}

#[test]
fn emits_microbial_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G207".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h9".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "scientific-invalidity".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "microbial boundary advisory warning".to_string(),
        recommendation: "resolve readiness mismatches before scientific evaluation".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "microbial_corpus_trim_align_call_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "microbial_corpus_findings_count=1"));
}

#[test]
fn emits_sv_boundary_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G208".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-1501".to_string(),
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
        .any(|check| check == "sv_boundary_coverage_call_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "sv_boundary_non_info_queue_entries=1"));
}

#[test]
fn emits_cnv_boundary_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G209".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "CNV boundary refuse anchor row".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "cnv_boundary_coverage_filter_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "cnv_boundary_findings_count=1"));
}

#[test]
fn emits_cram_boundary_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G210".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-1601".to_string(),
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
        .any(|check| check == "cram_boundary_validate_summary_filter_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "cram_boundary_queue_entries=1"));
}

#[test]
fn emits_long_read_boundary_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G211".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h6".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "readiness-degraded".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "long-read align stage is degraded".to_string(),
        recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "long_read_boundary_validate_align_summary_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "long_read_boundary_findings_count=1"));
}

#[test]
fn emits_sample_swap_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G212".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h11".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "scientific-caveat".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "sample-swap caveat surfaced in stats stage".to_string(),
        recommendation: "gate downstream interpretation on swap review".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "sample_swap_profile_validate_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "sample_swap_findings_count=1"));
}

#[test]
fn emits_duplicate_cohort_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G213".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "reproducibility".to_string(),
        row_id: "h7".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "reproducibility-low-repeats".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "duplicate cohort coverage stage requires extra repeats".to_string(),
        recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "duplicate_cohort_profile_coverage_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "duplicate_cohort_findings_count=1"));
}

#[test]
fn emits_panel_choice_sensitivity_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G214".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "database-compatibility".to_string(),
        row_id: "h12".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "database-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "panel-choice imputation boundary needs map compatibility review".to_string(),
        recommendation: "align panel and map bundle with selected reference".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "panel_choice_impute_call_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "panel_choice_findings_count=1"));
}

#[test]
fn emits_annotation_drift_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G215".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "scientific-caveat".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "annotation source drift changed postprocess expectations".to_string(),
        recommendation: "review annotation source provenance and caveat scope".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "annotation_drift_call_filter_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "annotation_drift_findings_count=1"));
}

#[test]
fn emits_qc_threshold_sensitivity_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G216".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "failure-class".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "QC-threshold boundary triggers refuse path".to_string(),
        recommendation: "resolve threshold gate reasons and re-run appraisal".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "qc_threshold_trim_coverage_filter_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "qc_threshold_findings_count=1"));
}

#[test]
fn emits_artifact_minimization_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G217".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "code-freeze".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "code-freeze-incomplete".to_string(),
        result_scope: "encrypted-code".to_string(),
        summary: "artifact minimization bundle misses required provenance metadata".to_string(),
        recommendation: "include provenance lock and script digest entries".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "artifact_minimization_profile_summary_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "artifact_minimization_code_scope_findings=1"));
}

#[test]
fn emits_shared_storage_io_stress_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G218".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-1701".to_string(),
        severity: "warning".to_string(),
        failure_class: "runtime-outlier".to_string(),
        recommendation: "review shared-storage IO staging and scratch placement".to_string(),
        affected_rows: vec!["h3".to_string()],
        source_appraisers: vec!["runtime-profile".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "shared_storage_io_profile_summary_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "shared_storage_io_queue_entries=1"));
}

#[test]
fn emits_database_build_benchmark_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G219".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h12".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "database-build benchmark impute stage has runtime outlier".to_string(),
        recommendation: "profile database build assets and rebuild expensive indexes".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "database_build_align_call_impute_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "database_build_findings_count=1"));
}

#[test]
fn emits_truth_set_anchored_corpus_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G220".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "readiness-refuse".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "truth-set anchored boundary hits filter refuse row".to_string(),
        recommendation: "resolve refusal causes before truth-set scoring".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "truth_set_validate_call_filter_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "truth_set_findings_count=1"));
}
