//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted local VCF imputation-metrics smoke evidence.

use anyhow::{bail, Result};

use crate::model::local_vcf_imputation_metrics_smoke::{
    BenchmarkLocalVcfImputationMetricsSmokeReport,
    BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1,
};

const STAGE_ID: &str = "vcf.imputation_metrics";
const COMPLETE: &str = "complete";
const EXPLICIT_MISSING: &str = "explicit_missing_quality_fields";

/// Validates the semantic closure of a local VCF imputation-metrics smoke record.
///
/// # Errors
///
/// Returns an error when identities, timing, metrics, availability flags, missing-field closure,
/// or status do not satisfy the persisted contract.
pub fn validate_benchmark_local_vcf_imputation_metrics_smoke_report(
    report: &BenchmarkLocalVcfImputationMetricsSmokeReport,
) -> Result<()> {
    if report.schema_version != BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1 {
        bail!("unsupported local VCF imputation-metrics smoke schema");
    }
    if report.stage_id != STAGE_ID {
        bail!("local VCF imputation-metrics report has the wrong stage identity");
    }
    for (name, value) in [
        ("command", report.command.as_str()),
        ("tool_id", report.tool_id.as_str()),
        ("corpus_id", report.corpus_id.as_str()),
        ("input_fixture_id", report.input_fixture_id.as_str()),
        ("panel_id", report.panel_id.as_str()),
        ("map_id", report.map_id.as_str()),
        ("output_root", report.output_root.as_str()),
        ("imputation_metrics_path", report.imputation_metrics_path.as_str()),
        ("source_imputation_qc_path", report.source_imputation_qc_path.as_str()),
        ("source_impute_smoke_metrics_path", report.source_impute_smoke_metrics_path.as_str()),
        ("source_imputation_manifest_path", report.source_imputation_manifest_path.as_str()),
        ("stage_result_manifest_path", report.stage_result_manifest_path.as_str()),
        ("started_at", report.started_at.as_str()),
        ("finished_at", report.finished_at.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!("local VCF imputation-metrics report has empty {name}");
        }
    }
    if !report.elapsed_seconds.is_finite() || report.elapsed_seconds < 0.0 {
        bail!("local VCF imputation-metrics report has invalid elapsed time");
    }
    if report.exit_code != 0 {
        bail!("successful local VCF imputation-metrics report requires exit code zero");
    }
    for (name, value) in [
        ("concordance", report.concordance),
        ("mean_info_score", Some(report.mean_info_score)),
        ("dosage_r2", report.dosage_r2),
    ] {
        if value.is_some_and(|metric| !metric.is_finite() || !(0.0..=1.0).contains(&metric)) {
            bail!("local VCF imputation-metrics report has invalid {name}");
        }
    }
    if report.r2_available != report.dosage_r2.is_some()
        || report.quality_field_availability.concordance != report.concordance.is_some()
        || report.quality_field_availability.dosage_r2 != report.dosage_r2.is_some()
    {
        bail!("local VCF imputation-metrics availability flags drift from metrics");
    }
    let expected_missing = [
        ("concordance", report.quality_field_availability.concordance),
        ("dosage_r2", report.quality_field_availability.dosage_r2),
        ("maf_strata", report.quality_field_availability.maf_strata),
        ("masked_truth_sites", report.quality_field_availability.masked_truth_sites),
    ]
    .into_iter()
    .filter_map(|(field, available)| (!available).then_some(field))
    .collect::<Vec<_>>();
    if report.missing_quality_fields.iter().map(String::as_str).collect::<Vec<_>>()
        != expected_missing
    {
        bail!("local VCF imputation-metrics missing-field closure drifted");
    }
    let expected_status = if expected_missing.is_empty() { COMPLETE } else { EXPLICIT_MISSING };
    if report.status != expected_status {
        bail!("local VCF imputation-metrics status disagrees with missing fields");
    }
    Ok(())
}
