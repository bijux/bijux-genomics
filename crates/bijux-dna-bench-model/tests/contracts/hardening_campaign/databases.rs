//! Owner: original hardening databases regression scenarios.
use super::*;

#[test]
fn emits_corpus_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G173".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "corpus-suitability".to_string(),
        row_id: "h3".to_string(),
        severity: "warning".to_string(),
        confidence: "high".to_string(),
        failure_class: "corpus-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "corpus profile mismatch".to_string(),
        recommendation: "materialize corpus profile matching stage scientific claim".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "corpus_hardening_profile_stage_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "corpus_hardening_corpus_mismatch_findings=1"));
}

#[test]
fn emits_database_hardening_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G174".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "database_hardening_alignment_and_call_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "database_hardening_reference_panel_bound=false"));
}

#[test]
fn emits_known_sites_bundle_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G221".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "database-compatibility".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "database-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "known-sites bundle mismatch at filter boundary".to_string(),
        recommendation: "align known-sites bundle with reference build".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "known_sites_validate_filter_postprocess_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "known_sites_findings_count=1"));
}

#[test]
fn emits_imputation_panel_bundle_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G222".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "database-compatibility".to_string(),
        row_id: "h12".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "database-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "imputation panel map trust label mismatch".to_string(),
        recommendation: "align panel-map trust profile before campaign promotion".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "imputation_panel_call_impute_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "imputation_panel_findings_count=1"));
}

#[test]
fn emits_taxonomy_variant_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G223".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "corpus-suitability".to_string(),
        row_id: "h3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "corpus-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "taxonomy variant profile requires rank-resolution caveat".to_string(),
        recommendation: "bind taxonomy variant profile to intended rank boundary".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "taxonomy_variant_profile_contam_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "taxonomy_variant_findings_count=1"));
}

#[test]
fn emits_contaminant_variant_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G224".to_string()];
    let queue = vec![HardeningQueueEntry {
        queue_id: "hardening-1801".to_string(),
        severity: "critical".to_string(),
        failure_class: "readiness-refuse".to_string(),
        recommendation: "resolve contaminant boundary refusal reasons".to_string(),
        affected_rows: vec!["h10".to_string()],
        source_appraisers: vec!["failure-class".to_string()],
    }];
    let entries = build_goal_entries(&selected, &matrix, &[], &queue);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "contaminant_variant_validate_contam_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "contaminant_variant_queue_entries=1"));
}

#[test]
fn emits_adapter_primer_variant_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G225".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h3".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "scientific-caveat".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "adapter-primer variant raises profile caveat".to_string(),
        recommendation: "review platform/assay primer bank selection".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "adapter_primer_trim_profile_contam_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "adapter_primer_findings_count=1"));
}

#[test]
fn emits_annotation_db_variant_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G226".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h13".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "scientific-caveat".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "annotation DB variant changes postprocess caveat".to_string(),
        recommendation: "review annotation DB provenance and caveat policy".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "annotation_db_call_filter_postprocess_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "annotation_db_findings_count=1"));
}

#[test]
fn emits_corpus_db_compatibility_matrix_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G227".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "database-compatibility".to_string(),
        row_id: "h5".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "database-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "corpus/database matrix mismatch for align boundary".to_string(),
        recommendation: "update compatibility matrix for selected corpus and DB bundle".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "corpus_db_matrix_validate_align_call_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "corpus_db_matrix_findings_count=1"));
}
