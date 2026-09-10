//! imputation metrics observations variants.
use crate::imputation_metrics::model::ImputationMetricsSummary;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkImputationQualityFieldAvailability as QualityFieldAvailability;

pub(crate) fn summarize_imputation_metrics(
    imputation_qc: &serde_json::Value,
) -> Result<ImputationMetricsSummary> {
    let concordance = imputation_qc
        .pointer("/concordance/genotype_concordance")
        .and_then(serde_json::Value::as_f64);
    let dosage_r2 =
        imputation_qc.pointer("/concordance/dosage_r2").and_then(serde_json::Value::as_f64);
    let maf_strata_present = imputation_qc
        .pointer("/concordance/maf_strata")
        .and_then(serde_json::Value::as_array)
        .is_some();
    let masked_truth_sites = imputation_qc
        .pointer("/concordance/masked_truth_site_count")
        .and_then(serde_json::Value::as_u64);
    let mean_info_score = imputation_qc
        .get("imputation_info_mean")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("missing `imputation_info_mean` in imputation QC payload"))?;
    let low_confidence_sites = imputation_qc
        .get("low_confidence_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing `low_confidence_count` in imputation QC payload"))?;

    let quality_field_availability = QualityFieldAvailability {
        concordance: concordance.is_some(),
        dosage_r2: dosage_r2.is_some(),
        maf_strata: maf_strata_present,
        masked_truth_sites: masked_truth_sites.is_some(),
    };
    let mut missing_quality_fields = Vec::<String>::new();
    if !quality_field_availability.concordance {
        missing_quality_fields.push("concordance".to_string());
    }
    if !quality_field_availability.dosage_r2 {
        missing_quality_fields.push("dosage_r2".to_string());
    }
    if !quality_field_availability.maf_strata {
        missing_quality_fields.push("maf_strata".to_string());
    }
    if !quality_field_availability.masked_truth_sites {
        missing_quality_fields.push("masked_truth_sites".to_string());
    }
    let status = if missing_quality_fields.is_empty() {
        "complete".to_string()
    } else {
        "explicit_missing_quality_fields".to_string()
    };

    Ok(ImputationMetricsSummary {
        concordance,
        mean_info_score,
        r2_available: dosage_r2.is_some(),
        dosage_r2,
        low_confidence_sites,
        masked_truth_sites: masked_truth_sites.unwrap_or(0),
        quality_field_availability,
        missing_quality_fields,
        status,
    })
}
