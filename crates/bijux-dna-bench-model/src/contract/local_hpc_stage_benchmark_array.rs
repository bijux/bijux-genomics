//! Semantic admission for persisted local HPC stage-benchmark arrays.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalHpcStageBenchmarkArrayReport,
    BenchmarkLocalHpcStageBenchmarkArrayRow, BENCHMARK_LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_V1,
};

use super::local_hpc_array::{parse_time, validate_resources};

/// Validate array identity, execution resolution, paths, commands, and resource envelope.
///
/// # Errors
/// Returns an error when a persisted stage-benchmark array is inconsistent.
pub fn validate_benchmark_local_hpc_stage_benchmark_array(
    report: &BenchmarkLocalHpcStageBenchmarkArrayReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_V1 {
        return invalid("unsupported local HPC stage-benchmark array schema");
    }
    for (value, field) in [
        (&report.script_path, "script_path"),
        (&report.manifest_path, "manifest_path"),
        (&report.wrapper_log_root, "wrapper_log_root"),
    ] {
        require_path(value, field)?;
    }
    if report.rows.is_empty()
        || report.benchmark_job_count != report.rows.len()
        || report.array_spec != format!("0-{}", report.rows.len() - 1)
    {
        return invalid("local HPC stage-benchmark array aggregate does not close");
    }
    let mut jobs = BTreeSet::new();
    let mut results = BTreeSet::new();
    for (index, row) in report.rows.iter().enumerate() {
        if row.array_index != index
            || !jobs.insert(row.job_id_local.as_str())
            || !results.insert(row.result_id.as_str())
        {
            return invalid("local HPC stage-benchmark array identity or ordering drifted");
        }
        validate_row(row)?;
    }
    let max_time = report
        .rows
        .iter()
        .map(|row| parse_time(&row.resources.time_limit))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max()
        .ok_or_else(|| BenchError::InvalidPolicy("empty resource envelope".to_string()))?;
    let envelope_time = parse_time(&report.resource_envelope.time_limit)?;
    if report.resource_envelope.cpus_per_task
        != report.rows.iter().map(|row| row.resources.cpus_per_task).max().unwrap_or(0)
        || report.resource_envelope.memory_mb
            != report.rows.iter().map(|row| row.resources.memory_mb).max().unwrap_or(0)
        || report.resource_envelope.scratch_gb
            != report.rows.iter().map(|row| row.resources.scratch_gb).max().unwrap_or(0)
        || envelope_time != max_time
    {
        return invalid("local HPC stage-benchmark resource envelope drifted");
    }
    Ok(())
}

fn validate_row(row: &BenchmarkLocalHpcStageBenchmarkArrayRow) -> Result<(), BenchError> {
    for (value, field) in [
        (&row.job_id_local, "rows[].job_id_local"),
        (&row.result_id, "rows[].result_id"),
        (&row.domain, "rows[].domain"),
        (&row.stage_id, "rows[].stage_id"),
        (&row.tool_id, "rows[].tool_id"),
        (&row.corpus_id, "rows[].corpus_id"),
        (&row.asset_profile_id, "rows[].asset_profile_id"),
        (&row.execution_mode, "rows[].execution_mode"),
        (&row.execution_mode_summary, "rows[].execution_mode_summary"),
        (&row.resolution_kind, "rows[].resolution_kind"),
        (&row.benchmark_result_command, "rows[].benchmark_result_command"),
    ] {
        require_text(value, field)?;
    }
    if !matches!(row.domain.as_str(), "fastq" | "bam" | "vcf")
        || row.benchmark_result_argv.is_empty()
        || row.output_paths.is_empty()
    {
        return invalid("local HPC stage-benchmark row scope or command is incomplete");
    }
    let unavailable = row.resolution_kind == "unavailable_with_reason";
    let resolution_valid = if unavailable {
        row.resolution_target.is_empty()
            && row.unavailable_reason.as_deref().is_some_and(|reason| !reason.trim().is_empty())
    } else {
        !row.resolution_target.trim().is_empty() && row.unavailable_reason.is_none()
    };
    if !resolution_valid {
        return invalid("local HPC stage-benchmark execution resolution is inconsistent");
    }
    for (value, field) in [
        (&row.script_path, "rows[].script_path"),
        (&row.scratch_root, "rows[].scratch_root"),
        (&row.input_links_root, "rows[].input_links_root"),
        (&row.output_root, "rows[].output_root"),
        (&row.log_root, "rows[].log_root"),
        (&row.stdout_path, "rows[].stdout_path"),
        (&row.stderr_path, "rows[].stderr_path"),
    ] {
        require_path(value, field)?;
    }
    for output in &row.output_paths {
        require_path(output, "rows[].output_paths[]")?;
    }
    validate_resources(&row.resources)
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local HPC stage-benchmark array requires non-empty {field}"));
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
            "local HPC stage-benchmark array requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
