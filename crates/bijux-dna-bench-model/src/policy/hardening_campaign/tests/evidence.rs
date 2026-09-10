//! Owner: original hardening evidence regression scenarios.
use super::*;

#[test]
fn emits_flagship_dashboard_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G251".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h11".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "scientific-invalidity".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "flagship dashboard import shows scientific fail row in stats stage".to_string(),
        recommendation: "highlight pass/fail and appraiser scientific status in dashboard"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "flagship_dashboard_profile_summary_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "flagship_dashboard_encrypted_scope_findings=1"));
}

#[test]
fn emits_stage_dossier_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G252".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "scientific-invalidity".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "stage dossier import includes refused filter row for scientific caveat"
            .to_string(),
        recommendation: "expose failed stage caveat details in per-stage dossier output"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "stage_dossier_validate_validate_filter_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "stage_dossier_refuse_rows=1"));
}

#[test]
fn emits_tool_dossier_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G253".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "artifact-validity".to_string(),
        row_id: "h5".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "missing-tool-binding".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "tool dossier import reports missing tool-binding around align stage".to_string(),
        recommendation: "record tool support and binding caveats in per-tool dossier output"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "tool_dossier_trim_align_call_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "tool_dossier_findings_count=1"));
}

#[test]
fn emits_corpus_dossier_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G254".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "corpus-suitability".to_string(),
        row_id: "h10".to_string(),
        severity: "warning".to_string(),
        confidence: "high".to_string(),
        failure_class: "corpus-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "corpus dossier import includes corpus mismatch at filter boundary".to_string(),
        recommendation: "record corpus suitability weaknesses and mismatch caveats per stage"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "corpus_dossier_profile_coverage_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "corpus_dossier_corpus_mismatch_findings=1"));
}

#[test]
fn emits_database_dossier_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G255".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "database-compatibility".to_string(),
        row_id: "h12".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "database-mismatch".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "database dossier import reports panel mismatch at impute boundary".to_string(),
        recommendation: "record source compatibility and update impact in database dossiers"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "database_dossier_align_impute_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "database_dossier_mismatch_findings=1"));
}

#[test]
fn emits_image_dossier_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G256".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h6".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "image dossier import reports runtime drift at mapping-summary stage".to_string(),
        recommendation: "track smoke and drift outcomes in per-image dossier output".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "image_dossier_trim_summary_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "image_dossier_drift_findings=1"));
}

#[test]
fn emits_stage_promotion_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G257".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "stage_promotion_validate_validate_stats_bound=true"));
    assert!(entries[0].goal_checks.iter().any(|check| check == "stage_promotion_pass_gate=true"));
}

#[test]
fn emits_stage_demotion_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G258".to_string()];
    let entries = build_goal_entries(&selected, &matrix, &[], &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "stage_demotion_profile_contamination_filter_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "stage_demotion_requires_demotion=true"));
}

#[test]
fn emits_backend_selection_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G259".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h5".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "backend selection import reports runtime drift around align stage".to_string(),
        recommendation: "prefer backend defaults with stable runtime evidence for scenario"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "backend_selection_trim_align_call_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "backend_selection_runtime_repro_findings=1"));
}

#[test]
fn emits_slurm_resource_default_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G260".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-performance".to_string(),
        row_id: "h7".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-under-sampled".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "resource-default import reports under-sampled runtime at coverage stage"
            .to_string(),
        recommendation: "raise baseline Slurm defaults from observed usage intervals".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "slurm_resource_coverage_summary_postprocess_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "slurm_resource_runtime_findings=1"));
}

#[test]
fn emits_methods_appendix_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G279".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "runtime-profile".to_string(),
        row_id: "h6".to_string(),
        severity: "warning".to_string(),
        confidence: "medium".to_string(),
        failure_class: "runtime-outlier".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "methods appendix import captures runtime envelope drift evidence".to_string(),
        recommendation: "derive methods/runtime appendix directly from benchmark evidence"
            .to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "methods_appendix_profile_summary_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "methods_appendix_runtime_repro_findings=1"));
}

#[test]
fn emits_public_caveat_checks() {
    let matrix = matrix_fixture();
    let selected = vec!["G280".to_string()];
    let findings = vec![AppraisalFinding {
        appraiser_id: "scientific-output".to_string(),
        row_id: "h10".to_string(),
        severity: "critical".to_string(),
        confidence: "high".to_string(),
        failure_class: "scientific-invalidity".to_string(),
        result_scope: "encrypted-results".to_string(),
        summary: "public caveat import captures scientific-invalidity at filter stage".to_string(),
        recommendation: "publish tested/weak/simulated caveats from benchmark findings".to_string(),
    }];
    let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
    assert_eq!(entries.len(), 1);
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "public_caveat_contamination_filter_stats_bound=true"));
    assert!(entries[0]
        .goal_checks
        .iter()
        .any(|check| check == "public_caveat_summary_required=true"));
}
