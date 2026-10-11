//! Admission invariants for stage-tool benchmark runtime declarations.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BenchError, StageToolContainerRow, StageToolContainersConfig, StageToolContainersReport,
    LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1, STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN,
    STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_V1, STAGE_TOOL_CONTAINERS_SCOPE,
};

/// Validates a persisted stage-tool runtime declaration configuration.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for invalid identity, ordering, or runtime shape.
pub fn validate_stage_tool_containers_config(
    config: &StageToolContainersConfig,
) -> Result<(), BenchError> {
    if config.schema_version != LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1
        || config.classification_scope != STAGE_TOOL_CONTAINERS_SCOPE
    {
        return invalid("stage-tool runtime config identity is invalid");
    }
    validate_rows(&config.rows)
}

/// Validates a persisted stage-tool runtime readiness report.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for invalid identity, path, rows, or aggregates.
pub fn validate_stage_tool_containers_report(
    report: &StageToolContainersReport,
) -> Result<(), BenchError> {
    if report.schema_version != STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_V1
        || report.classification_scope != STAGE_TOOL_CONTAINERS_SCOPE
        || !safe_relative_path(&report.config_path)
    {
        return invalid("stage-tool runtime report identity or path is invalid");
    }
    validate_rows(&report.rows)?;
    if report.row_count != report.rows.len()
        || report.benchmark_ready_row_count != report.rows.len()
        || report.external_row_count
            != report.rows.iter().filter(|row| row.install_kind != "workspace_binary").count()
        || report.container_declared_row_count
            != report.rows.iter().filter(|row| row.container_id.is_some()).count()
        || report.command_entrypoint_row_count
            != report.rows.iter().filter(|row| row.command_entrypoint.is_some()).count()
        || report.host_binary_row_count
            != report.rows.iter().filter(|row| row.host_binary_mode.is_some()).count()
        || report.domain_counts != count_by(&report.rows, |row| row.domain.as_str())
        || report.execution_mode_counts != count_by(&report.rows, |row| row.execution_mode.as_str())
    {
        return invalid("stage-tool runtime report aggregates are inconsistent");
    }
    Ok(())
}

fn validate_rows(rows: &[StageToolContainerRow]) -> Result<(), BenchError> {
    if rows.is_empty() {
        return invalid("stage-tool runtime rows are empty");
    }
    let mut identities = BTreeSet::new();
    for row in rows {
        if !matches!(row.domain.as_str(), "bam" | "fastq" | "vcf")
            || !row.stage_id.starts_with(&format!("{}.", row.domain))
            || [
                row.stage_id.as_str(),
                row.tool_id.as_str(),
                row.execution_mode.as_str(),
                row.install_kind.as_str(),
                row.declaration_origin.as_str(),
            ]
            .iter()
            .any(|value| value.trim().is_empty())
            || row.declaration_origin != STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN
            || optional_blank(row.container_id.as_deref())
            || optional_blank(row.command_entrypoint.as_deref())
            || optional_blank(row.host_binary_mode.as_deref())
            || !runtime_shape_is_valid(row)
            || !identities.insert((
                row.domain.as_str(),
                row.stage_id.as_str(),
                row.tool_id.as_str(),
            ))
        {
            return invalid("stage-tool runtime row semantics are invalid");
        }
    }
    if rows.windows(2).any(|rows| row_key(&rows[0]) >= row_key(&rows[1])) {
        return invalid("stage-tool runtime rows are not canonically ordered");
    }
    Ok(())
}

fn runtime_shape_is_valid(row: &StageToolContainerRow) -> bool {
    match row.install_kind.as_str() {
        "workspace_binary" => {
            row.host_binary_mode.as_deref() == Some("workspace_binary")
                && row.command_entrypoint.is_some()
                && row.container_id.is_none()
        }
        "container" => {
            row.container_id.as_deref().is_some_and(concrete_container_identity)
                && row.command_entrypoint.is_some()
                && row.host_binary_mode.is_none()
        }
        "external" => {
            row.execution_mode == "external"
                && row.container_id.as_deref().is_some_and(|id| {
                    id.strip_prefix("external-artifact://") == Some(row.tool_id.as_str())
                })
                && row.command_entrypoint.is_some()
                && row.host_binary_mode.is_none()
        }
        _ => false,
    }
}

fn concrete_container_identity(value: &str) -> bool {
    !value.split(['@', '/']).any(|segment| {
        matches!(
            segment.trim().to_ascii_lowercase().as_str(),
            "external" | "planned" | "todo" | "unknown"
        )
    })
}

fn optional_blank(value: Option<&str>) -> bool {
    value.is_some_and(|value| value.trim().is_empty())
}

fn row_key(row: &StageToolContainerRow) -> (&str, &str, &str) {
    (&row.domain, &row.stage_id, &row.tool_id)
}

fn count_by<'a>(
    rows: &'a [StageToolContainerRow],
    key: impl Fn(&'a StageToolContainerRow) -> &'a str,
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
