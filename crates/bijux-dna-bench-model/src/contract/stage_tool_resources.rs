//! Admission invariants for stage-tool benchmark resource contracts.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BenchError, StageToolResourceRow, StageToolResourcesConfig, StageToolResourcesReport,
    BAM_STAGE_TOOL_RESOURCE_ORIGIN, FASTQ_STAGE_TOOL_RESOURCE_ORIGIN,
    LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1, STAGE_TOOL_RESOURCES_REPORT_SCHEMA_V1,
    STAGE_TOOL_RESOURCES_SCOPE, VCF_STAGE_TOOL_RESOURCE_ORIGIN,
};

/// Validates a persisted stage-tool resource configuration.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] when identity, ordering, scope, or resource
/// invariants are inconsistent.
pub fn validate_stage_tool_resources_config(
    config: &StageToolResourcesConfig,
) -> Result<(), BenchError> {
    if config.schema_version != LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1
        || config.classification_scope != STAGE_TOOL_RESOURCES_SCOPE
    {
        return invalid("stage-tool resource config identity is invalid");
    }
    validate_rows(&config.rows)
}

/// Validates a persisted stage-tool resource readiness report.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] when identity, path, rows, or aggregate closure is
/// inconsistent.
pub fn validate_stage_tool_resources_report(
    report: &StageToolResourcesReport,
) -> Result<(), BenchError> {
    if report.schema_version != STAGE_TOOL_RESOURCES_REPORT_SCHEMA_V1
        || report.classification_scope != STAGE_TOOL_RESOURCES_SCOPE
        || !safe_relative_path(&report.config_path)
    {
        return invalid("stage-tool resource report identity or path is invalid");
    }
    validate_rows(&report.rows)?;
    if report.row_count != report.rows.len()
        || report.benchmark_ready_row_count != report.rows.len()
        || report.nonzero_resource_row_count != report.rows.len()
        || report.domain_counts != count_by(&report.rows, |row| row.domain.as_str())
        || report.resource_origin_counts
            != count_by(&report.rows, |row| row.resource_origin.as_str())
    {
        return invalid("stage-tool resource report aggregates are inconsistent");
    }
    Ok(())
}

fn validate_rows(rows: &[StageToolResourceRow]) -> Result<(), BenchError> {
    if rows.is_empty() {
        return invalid("stage-tool resource rows are empty");
    }
    let mut identities = BTreeSet::new();
    for row in rows {
        if !matches!(row.domain.as_str(), "bam" | "fastq" | "vcf")
            || [row.stage_id.as_str(), row.tool_id.as_str(), row.resource_origin.as_str()]
                .iter()
                .any(|value| value.trim().is_empty())
            || ![
                BAM_STAGE_TOOL_RESOURCE_ORIGIN,
                FASTQ_STAGE_TOOL_RESOURCE_ORIGIN,
                VCF_STAGE_TOOL_RESOURCE_ORIGIN,
            ]
            .contains(&row.resource_origin.as_str())
            || row.threads == 0
            || row.memory_gb == 0
            || row.walltime_minutes == 0
            || row.scratch_gb == 0
            || !identities.insert((
                row.domain.as_str(),
                row.stage_id.as_str(),
                row.tool_id.as_str(),
            ))
        {
            return invalid("stage-tool resource row semantics are invalid");
        }
    }
    if rows.windows(2).any(|rows| row_key(&rows[0]) >= row_key(&rows[1])) {
        return invalid("stage-tool resource rows are not canonically ordered");
    }
    Ok(())
}

fn row_key(row: &StageToolResourceRow) -> (&str, &str, &str) {
    (row.domain.as_str(), row.stage_id.as_str(), row.tool_id.as_str())
}

fn count_by<'a>(
    rows: &'a [StageToolResourceRow],
    key: impl Fn(&'a StageToolResourceRow) -> &'a str,
) -> BTreeMap<String, usize> {
    rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(key(row).to_string()).or_default() += 1;
        counts
    })
}

fn safe_relative_path(value: &str) -> bool {
    let path = std::path::Path::new(value);
    !value.trim().is_empty()
        && !path.is_absolute()
        && path.components().all(|component| {
            matches!(component, std::path::Component::Normal(_) | std::path::Component::CurDir)
        })
}

fn invalid<T>(reason: &str) -> Result<T, BenchError> {
    Err(BenchError::InvalidObservation { reason: reason.to_string() })
}
