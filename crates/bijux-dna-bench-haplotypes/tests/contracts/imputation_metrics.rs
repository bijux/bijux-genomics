//! Original scientific and binding behavior at its owning capability.
use crate::imputation_metrics::observations::variants::summarize_imputation_metrics;
#[test]
fn imputation_metrics_summary_reports_missing_quality_fields_explicitly() {
    let qc = serde_json::json!({
        "imputation_info_mean": 0.81,
        "low_confidence_count": 3,
        "concordance": {
            "genotype_concordance": serde_json::Value::Null,
            "dosage_r2": serde_json::Value::Null
        }
    });
    let summary = summarize_imputation_metrics(&qc).expect("summarize quality payload");
    assert_eq!(summary.concordance, None);
    assert!(!summary.r2_available);
    assert_eq!(summary.masked_truth_sites, 0);
    assert_eq!(
        summary.missing_quality_fields,
        vec![
            "concordance".to_string(),
            "dosage_r2".to_string(),
            "maf_strata".to_string(),
            "masked_truth_sites".to_string()
        ]
    );
    assert_eq!(summary.status, "explicit_missing_quality_fields");
}
