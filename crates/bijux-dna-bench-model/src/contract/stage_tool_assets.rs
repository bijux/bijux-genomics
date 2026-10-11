//! Admission invariants for stage-tool benchmark asset contracts.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BenchError, StageToolAssetRow, StageToolAssetsConfig, StageToolAssetsReport,
    LOCAL_STAGE_TOOL_ASSETS_SCHEMA_V1, STAGE_TOOL_ASSETS_DECLARATION_ORIGIN,
    STAGE_TOOL_ASSETS_REPORT_SCHEMA_V1, STAGE_TOOL_ASSETS_SCOPE,
};

/// Validates a persisted stage-tool asset configuration.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for invalid identity, paths, ordering, or semantics.
pub fn validate_stage_tool_assets_config(config: &StageToolAssetsConfig) -> Result<(), BenchError> {
    if config.schema_version != LOCAL_STAGE_TOOL_ASSETS_SCHEMA_V1
        || config.classification_scope != STAGE_TOOL_ASSETS_SCOPE
    {
        return invalid("stage-tool asset config identity is invalid");
    }
    validate_rows(&config.rows)
}

/// Validates a persisted stage-tool asset readiness report.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for invalid identity, paths, rows, or aggregates.
pub fn validate_stage_tool_assets_report(report: &StageToolAssetsReport) -> Result<(), BenchError> {
    if report.schema_version != STAGE_TOOL_ASSETS_REPORT_SCHEMA_V1
        || report.classification_scope != STAGE_TOOL_ASSETS_SCOPE
        || !safe_relative_path(&report.config_path)
    {
        return invalid("stage-tool asset report identity or path is invalid");
    }
    validate_rows(&report.rows)?;
    let stage_tools = report
        .rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str(), row.tool_id.as_str()))
        .collect::<BTreeSet<_>>();
    let asset_ids = report.rows.iter().map(|row| row.asset_id.as_str()).collect::<BTreeSet<_>>();
    if report.row_count != report.rows.len()
        || report.declared_stage_tool_row_count != stage_tools.len()
        || report.asset_id_row_count != report.rows.len()
        || report.unique_asset_id_count != asset_ids.len()
        || report.domain_counts != count_by(&report.rows, |row| row.domain.as_str())
        || report.asset_role_counts != count_by(&report.rows, |row| row.asset_role.as_str())
    {
        return invalid("stage-tool asset report aggregates are inconsistent");
    }
    Ok(())
}

fn validate_rows(rows: &[StageToolAssetRow]) -> Result<(), BenchError> {
    if rows.is_empty() {
        return invalid("stage-tool asset rows are empty");
    }
    let mut identities = BTreeSet::new();
    for row in rows {
        if !matches!(row.domain.as_str(), "bam" | "fastq")
            || !row.stage_id.starts_with(&format!("{}.", row.domain))
            || [
                row.stage_id.as_str(),
                row.tool_id.as_str(),
                row.asset_role.as_str(),
                row.asset_id.as_str(),
                row.declaration_origin.as_str(),
            ]
            .iter()
            .any(|value| value.trim().is_empty())
            || row.declaration_origin != STAGE_TOOL_ASSETS_DECLARATION_ORIGIN
            || !safe_relative_path(&row.asset_path)
            || !identities.insert((
                row.domain.as_str(),
                row.stage_id.as_str(),
                row.tool_id.as_str(),
                row.asset_role.as_str(),
                row.asset_id.as_str(),
            ))
        {
            return invalid("stage-tool asset row semantics are invalid");
        }
    }
    if rows.windows(2).any(|rows| row_key(&rows[0]) >= row_key(&rows[1])) {
        return invalid("stage-tool asset rows are not canonically ordered");
    }
    Ok(())
}

fn row_key(row: &StageToolAssetRow) -> (&str, &str, &str, &str, &str) {
    (&row.domain, &row.stage_id, &row.tool_id, &row.asset_role, &row.asset_id)
}

fn count_by<'a>(
    rows: &'a [StageToolAssetRow],
    key: impl Fn(&'a StageToolAssetRow) -> &'a str,
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
