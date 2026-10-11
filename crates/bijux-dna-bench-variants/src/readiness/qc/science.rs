use super::{
    FLOAT_TOLERANCE, REQUIRED_EXCLUDED_SAMPLE_ID, REQUIRED_EXCLUDED_SAMPLE_MISSINGNESS,
    REQUIRED_EXCLUDED_VARIANT_ID, REQUIRED_EXCLUDED_VARIANT_MISSINGNESS,
    REQUIRED_HETEROZYGOUS_CALL_COUNT, REQUIRED_HET_HOM_RATIO, REQUIRED_HOMOZYGOUS_ALT_CALL_COUNT,
    REQUIRED_HWE_PVALUE_MEAN, REQUIRED_HWE_STATUS, REQUIRED_HWE_TESTED_VARIANT_COUNT,
    REQUIRED_MAF_OBSERVED_VARIANT_COUNT, REQUIRED_SAMPLE_MISSINGNESS_EXCLUSION_THRESHOLD,
    REQUIRED_VARIANT_MISSINGNESS_EXCLUSION_THRESHOLD,
};
use crate::qc::LocalVcfQcSmokeReport;
pub(super) fn qc_smoke_matches_governed_contract(report: &LocalVcfQcSmokeReport) -> bool {
    !report.input_vcf_path.trim().is_empty()
        && !report.output_root.trim().is_empty()
        && !report.qc_json_path.trim().is_empty()
        && !report.qc_summary_path.trim().is_empty()
        && !report.qc_tables_path.trim().is_empty()
        && !report.imputation_qc_path.trim().is_empty()
        && !report.warnings_path.trim().is_empty()
        && !report.qc_histograms_path.trim().is_empty()
        && !report.metrics_path.trim().is_empty()
        && !report.stage_result_manifest_path.trim().is_empty()
        && !report.sample_missingness.is_empty()
        && !report.variant_missingness.is_empty()
        && report.maf_summary.observed_variant_count == REQUIRED_MAF_OBSERVED_VARIANT_COUNT
        && report.heterozygosity.heterozygous_call_count == REQUIRED_HETEROZYGOUS_CALL_COUNT
        && report.heterozygosity.homozygous_alt_call_count == REQUIRED_HOMOZYGOUS_ALT_CALL_COUNT
        && approx_eq(
            report.heterozygosity.het_hom_ratio.unwrap_or_default(),
            REQUIRED_HET_HOM_RATIO,
        )
        && report.hwe_summary.tested_variant_count == REQUIRED_HWE_TESTED_VARIANT_COUNT
        && approx_eq(report.hwe_summary.pvalue_mean.unwrap_or_default(), REQUIRED_HWE_PVALUE_MEAN)
        && report.hwe_summary.status == REQUIRED_HWE_STATUS
        && approx_eq(
            report.sample_missingness_exclusion_threshold,
            REQUIRED_SAMPLE_MISSINGNESS_EXCLUSION_THRESHOLD,
        )
        && approx_eq(
            report.variant_missingness_exclusion_threshold,
            REQUIRED_VARIANT_MISSINGNESS_EXCLUSION_THRESHOLD,
        )
        && report.excluded_samples.len() == 1
        && report.excluded_variants.len() == 1
        && report.excluded_samples[0].sample_id == REQUIRED_EXCLUDED_SAMPLE_ID
        && approx_eq(report.excluded_samples[0].missingness, REQUIRED_EXCLUDED_SAMPLE_MISSINGNESS)
        && report.excluded_variants[0].variant_id == REQUIRED_EXCLUDED_VARIANT_ID
        && approx_eq(report.excluded_variants[0].missingness, REQUIRED_EXCLUDED_VARIANT_MISSINGNESS)
        && report.sample_missingness.iter().any(|row| row.sample_id == REQUIRED_EXCLUDED_SAMPLE_ID)
        && report
            .variant_missingness
            .iter()
            .any(|row| row.variant_id == REQUIRED_EXCLUDED_VARIANT_ID)
}
pub(super) fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= FLOAT_TOLERANCE
}
