//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted VCF stage catalogs and render reports.

use std::collections::BTreeSet;

use crate::{
    model::{LocalVcfStageCatalogConfig, LocalVcfStageCatalogReport, VcfStageCatalogRow},
    BenchError,
};

use super::{LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_V1, LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1};

/// Validate an authored local VCF stage-catalog declaration.
///
/// # Errors
/// Returns an error when schema identity, stage identity, support state, or row content is invalid.
pub fn validate_local_vcf_stage_catalog_config(
    config: &LocalVcfStageCatalogConfig,
) -> Result<(), BenchError> {
    if config.schema_version != LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1 {
        return invalid("unsupported local VCF stage-catalog schema");
    }
    validate_rows(&config.rows).map(|_| ())
}

/// Validate the report emitted while rendering the local VCF stage catalog.
///
/// # Errors
/// Returns an error when schema identity, row content, or aggregate counts do not close.
pub fn validate_local_vcf_stage_catalog_report(
    report: &LocalVcfStageCatalogReport,
) -> Result<(), BenchError> {
    if report.schema_version != LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_V1 {
        return invalid("unsupported local VCF stage-catalog report schema");
    }
    require_text(&report.config_path, "config_path")?;
    let counts = validate_rows(&report.rows)?;
    if report.stage_count != report.rows.len()
        || report.supported_stage_count != counts.supported
        || report.planned_stage_count != counts.planned
        || report.supported_stage_count + report.planned_stage_count != report.stage_count
    {
        return invalid("local VCF stage-catalog report aggregates do not close");
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct SupportCounts {
    supported: usize,
    planned: usize,
}

fn validate_rows(rows: &[VcfStageCatalogRow]) -> Result<SupportCounts, BenchError> {
    if rows.is_empty() {
        return invalid("VCF stage catalog has no rows");
    }
    let mut stages = BTreeSet::new();
    let mut supported = 0;
    let mut planned = 0;
    for row in rows {
        for (value, field) in [
            (row.stage_id.as_str(), "stage_id"),
            (row.stage_name.as_str(), "stage_name"),
            (row.default_tool_id.as_str(), "default_tool_id"),
            (row.metrics_schema_id.as_str(), "metrics_schema_id"),
            (row.benchmark_category.as_str(), "benchmark_category"),
            (row.local_smoke_mode.as_str(), "local_smoke_mode"),
        ] {
            require_text(value, field)?;
        }
        if !stages.insert(row.stage_id.as_str()) {
            return invalid("VCF stage catalog repeats a stage identity");
        }
        match row.support_status.as_str() {
            "supported" => supported += 1,
            "planned" => planned += 1,
            _ => return invalid("VCF stage catalog has an unsupported support_status"),
        }
        for (values, field) in
            [(&row.input_types, "input_types"), (&row.output_types, "output_types")]
        {
            if values.is_empty()
                || values.iter().any(|value| value.trim().is_empty())
                || values.iter().collect::<BTreeSet<_>>().len() != values.len()
            {
                return invalid(&format!("VCF stage catalog has invalid {field}"));
            }
        }
        if row.required_assets.iter().any(|value| value.trim().is_empty())
            || row.required_assets.iter().collect::<BTreeSet<_>>().len()
                != row.required_assets.len()
        {
            return invalid("VCF stage catalog has invalid required_assets");
        }
    }
    Ok(SupportCounts { supported, planned })
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(&format!("VCF stage catalog requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: &str) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.to_string()))
}
