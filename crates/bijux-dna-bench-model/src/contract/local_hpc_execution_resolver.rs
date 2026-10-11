//! Semantic admission for persisted local HPC execution-resolution reports.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalHpcExecutionResolverReport, BenchmarkLocalHpcExecutionResolverRow,
    BENCHMARK_LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_V1,
};

/// Validate execution identities, selected-job scope, resolution evidence, and exact counters.
///
/// # Errors
/// Returns an error when a resolution row or aggregate is incomplete or inconsistent.
pub fn validate_benchmark_local_hpc_execution_resolver(
    report: &BenchmarkLocalHpcExecutionResolverReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_V1 {
        return invalid("unsupported local HPC execution-resolver schema");
    }
    require_path(&report.output_path, "output_path")?;
    if Path::new(&report.output_path).extension() != Some(std::ffi::OsStr::new("tsv"))
        || report.rows.is_empty()
        || report.row_count != report.rows.len()
    {
        return invalid("local HPC execution-resolver aggregate does not close");
    }
    let mut prior_tool: Option<&str> = None;
    let mut tools = BTreeSet::new();
    for row in &report.rows {
        validate_row(row)?;
        if !tools.insert(row.tool_id.as_str())
            || prior_tool.is_some_and(|prior| prior >= row.tool_id.as_str())
        {
            return invalid("local HPC execution-resolver tool identities are not canonical");
        }
        prior_tool = Some(row.tool_id.as_str());
    }
    let mut counts = BTreeMap::new();
    for row in &report.rows {
        *counts.entry(row.resolution_kind.clone()).or_default() += 1;
    }
    if report.resolution_counts != counts
        || report.unclassified_tool_count
            != report.rows.iter().filter(|row| row.execution_mode == "unclassified").count()
    {
        return invalid("local HPC execution-resolver derived counters do not close");
    }
    Ok(())
}

fn validate_row(row: &BenchmarkLocalHpcExecutionResolverRow) -> Result<(), BenchError> {
    for (field, value) in [
        ("rows[].tool_id", row.tool_id.as_str()),
        ("rows[].lookup_tool_id", row.lookup_tool_id.as_str()),
        ("rows[].execution_mode", row.execution_mode.as_str()),
        ("rows[].execution_mode_summary", row.execution_mode_summary.as_str()),
        ("rows[].install_kind", row.install_kind.as_str()),
        ("rows[].resolution_kind", row.resolution_kind.as_str()),
    ] {
        require_text(value, field)?;
    }
    validate_sorted_unique(&row.domains, "rows[].domains")?;
    validate_sorted_unique(&row.selected_stage_ids, "rows[].selected_stage_ids")?;
    validate_sorted_unique(&row.selected_job_ids, "rows[].selected_job_ids")?;
    validate_sorted_unique_optional(&row.selected_result_ids, "rows[].selected_result_ids")?;
    validate_sorted_unique_optional(&row.selected_pipeline_ids, "rows[].selected_pipeline_ids")?;
    validate_sorted_unique_optional(&row.selected_node_ids, "rows[].selected_node_ids")?;
    validate_sorted_unique_optional(&row.runtime_probe_paths, "rows[].runtime_probe_paths")?;
    validate_sorted_unique_optional(&row.registry_paths, "rows[].registry_paths")?;
    if row.selected_job_count == 0 || row.selected_job_count != row.selected_job_ids.len() {
        return invalid("local HPC execution-resolver selected-job count does not close");
    }
    if row.domains.iter().any(|domain| !matches!(domain.as_str(), "fastq" | "bam" | "vcf"))
        || row.selected_stage_ids.iter().any(|stage| {
            stage
                .split_once('.')
                .is_none_or(|(domain, _)| !row.domains.iter().any(|value| value == domain))
        })
    {
        return invalid("local HPC execution-resolver domain and stage scope does not close");
    }
    for path in row.runtime_probe_paths.iter().chain(&row.registry_paths) {
        require_path(path, "rows[].evidence_paths[]")?;
    }
    if let Some(entrypoint) = row.command_entrypoint.as_deref() {
        require_text(entrypoint, "rows[].command_entrypoint")?;
    }
    match row.resolution_kind.as_str() {
        "host_binary" | "apptainer_image" => {
            require_text(&row.resolution_target, "rows[].resolution_target")?;
            if row.unavailable_reason.is_some() {
                return invalid("available execution resolution cannot carry unavailable_reason");
            }
        }
        "unavailable_with_reason" => {
            if !row.resolution_target.is_empty()
                || row.unavailable_reason.as_deref().map(str::trim).is_none_or(str::is_empty)
            {
                return invalid("unavailable execution resolution must carry only its reason");
            }
        }
        _ => return invalid("unsupported local HPC execution resolution kind"),
    }
    Ok(())
}

fn validate_sorted_unique(values: &[String], field: &str) -> Result<(), BenchError> {
    if values.is_empty() {
        return invalid(format!("local HPC execution resolver requires non-empty {field}"));
    }
    validate_sorted_unique_optional(values, field)
}

fn validate_sorted_unique_optional(values: &[String], field: &str) -> Result<(), BenchError> {
    let mut prior: Option<&str> = None;
    for value in values {
        require_text(value, field)?;
        if prior.is_some_and(|previous| previous >= value.as_str()) {
            return invalid(format!("local HPC execution resolver requires canonical {field}"));
        }
        prior = Some(value);
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local HPC execution resolver requires non-empty {field}"));
    }
    Ok(())
}

fn require_path(value: &str, field: &str) -> Result<(), BenchError> {
    require_text(value, field)?;
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!(
            "local HPC execution resolver requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
