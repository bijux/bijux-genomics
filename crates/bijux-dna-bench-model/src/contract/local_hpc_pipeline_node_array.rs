//! Semantic admission for persisted local HPC pipeline-node arrays.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalHpcArrayResources, BenchmarkLocalHpcPipelineNodeArrayDependency,
    BenchmarkLocalHpcPipelineNodeArrayExpectedOutput, BenchmarkLocalHpcPipelineNodeArrayReport,
    BenchmarkLocalHpcPipelineNodeArrayRow, BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1,
};

/// Validate array identity, dependency lineage, paths, commands, and resource envelope.
///
/// # Errors
/// Returns an error when a persisted pipeline-node array is inconsistent.
pub fn validate_benchmark_local_hpc_pipeline_node_array(
    report: &BenchmarkLocalHpcPipelineNodeArrayReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1 {
        return invalid("unsupported local HPC pipeline-node array schema");
    }
    for (value, field) in [
        (&report.script_path, "script_path"),
        (&report.manifest_path, "manifest_path"),
        (&report.wrapper_log_root, "wrapper_log_root"),
    ] {
        require_path(value, field)?;
    }
    if report.rows.is_empty()
        || report.pipeline_job_count != report.rows.len()
        || report.array_spec != format!("0-{}", report.rows.len() - 1)
    {
        return invalid("local HPC pipeline-node array aggregate does not close");
    }
    let dependency_count = report.rows.iter().map(|row| row.dependencies.len()).sum::<usize>();
    let pipeline_count =
        report.rows.iter().map(|row| row.pipeline_id.as_str()).collect::<BTreeSet<_>>().len();
    if report.dependency_count != dependency_count || report.pipeline_count != pipeline_count {
        return invalid("local HPC pipeline-node array count closure drifted");
    }

    let mut jobs = BTreeMap::new();
    let mut pipeline_nodes = BTreeSet::new();
    for (index, row) in report.rows.iter().enumerate() {
        if row.array_index != index
            || jobs.contains_key(row.job_id_local.as_str())
            || !pipeline_nodes.insert((row.pipeline_id.as_str(), row.node_id.as_str()))
        {
            return invalid("local HPC pipeline-node array identity or ordering drifted");
        }
        validate_row(row, &jobs)?;
        jobs.insert(row.job_id_local.as_str(), row);
    }
    validate_resource_envelope(report)
}

fn validate_row<'a>(
    row: &'a BenchmarkLocalHpcPipelineNodeArrayRow,
    prior_jobs: &BTreeMap<&'a str, &'a BenchmarkLocalHpcPipelineNodeArrayRow>,
) -> Result<(), BenchError> {
    for (value, field) in [
        (&row.job_id_local, "rows[].job_id_local"),
        (&row.pipeline_id, "rows[].pipeline_id"),
        (&row.node_id, "rows[].node_id"),
        (&row.domain, "rows[].domain"),
        (&row.stage_id, "rows[].stage_id"),
        (&row.tool_id, "rows[].tool_id"),
        (&row.corpus_id, "rows[].corpus_id"),
        (&row.asset_profile_id, "rows[].asset_profile_id"),
        (&row.pipeline_node_command, "rows[].pipeline_node_command"),
    ] {
        require_text(value, field)?;
    }
    if !matches!(row.domain.as_str(), "fastq" | "bam" | "vcf")
        || row.pipeline_node_argv.is_empty()
        || row.expected_outputs.is_empty()
    {
        return invalid("local HPC pipeline-node row scope, command, or outputs are incomplete");
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
    validate_outputs(&row.expected_outputs, "rows[].expected_outputs")?;
    validate_resources(&row.resources)?;

    if row.dependency_job_ids.len() != row.dependencies.len()
        || row
            .dependencies
            .iter()
            .zip(&row.dependency_job_ids)
            .any(|(dependency, job_id)| dependency.job_id_local != *job_id)
    {
        return invalid("local HPC pipeline-node dependency order or cardinality drifted");
    }
    let mut dependency_jobs = BTreeSet::new();
    let mut dependency_upstream_results = BTreeSet::new();
    for dependency in &row.dependencies {
        if !dependency_jobs.insert(dependency.job_id_local.as_str()) {
            return invalid("local HPC pipeline-node dependency identity is duplicated");
        }
        validate_dependency(row, dependency, prior_jobs)?;
        for result_id in &dependency.upstream_result_ids {
            if !dependency_upstream_results.insert(result_id.as_str()) {
                return invalid("local HPC pipeline-node upstream result ownership is duplicated");
            }
        }
    }
    let row_upstream_results =
        row.upstream_result_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if row_upstream_results.len() != row.upstream_result_ids.len()
        || row_upstream_results != dependency_upstream_results
    {
        return invalid("local HPC pipeline-node upstream result closure drifted");
    }
    Ok(())
}

fn validate_dependency<'a>(
    row: &BenchmarkLocalHpcPipelineNodeArrayRow,
    dependency: &BenchmarkLocalHpcPipelineNodeArrayDependency,
    prior_jobs: &BTreeMap<&'a str, &'a BenchmarkLocalHpcPipelineNodeArrayRow>,
) -> Result<(), BenchError> {
    require_text(&dependency.node_id, "rows[].dependencies[].node_id")?;
    require_text(&dependency.job_id_local, "rows[].dependencies[].job_id_local")?;
    let prior = prior_jobs.get(dependency.job_id_local.as_str()).ok_or_else(|| {
        BenchError::InvalidPolicy(
            "local HPC pipeline-node dependency must reference a prior row".to_string(),
        )
    })?;
    if dependency.array_index != prior.array_index
        || dependency.array_index >= row.array_index
        || dependency.node_id != prior.node_id
        || row.pipeline_id != prior.pipeline_id
    {
        return invalid("local HPC pipeline-node dependency lineage drifted");
    }
    validate_outputs(&dependency.expected_outputs, "rows[].dependencies[].expected_outputs")?;
    let prior_outputs = prior
        .expected_outputs
        .iter()
        .map(|output| (output.output_id.as_str(), output.output_path.as_str()))
        .collect::<BTreeSet<_>>();
    let dependency_outputs = dependency
        .expected_outputs
        .iter()
        .map(|output| (output.output_id.as_str(), output.output_path.as_str()))
        .collect::<BTreeSet<_>>();
    if !dependency_outputs.is_subset(&prior_outputs) {
        return invalid("local HPC pipeline-node dependency output ownership drifted");
    }
    let upstream =
        dependency.upstream_result_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if upstream.len() != dependency.upstream_result_ids.len()
        || (!upstream.is_empty()
            && upstream
                != dependency_outputs
                    .iter()
                    .map(|(output_id, _)| *output_id)
                    .collect::<BTreeSet<_>>())
    {
        return invalid("local HPC pipeline-node dependency result projection drifted");
    }
    Ok(())
}

fn validate_outputs(
    outputs: &[BenchmarkLocalHpcPipelineNodeArrayExpectedOutput],
    field: &str,
) -> Result<(), BenchError> {
    if outputs.is_empty() {
        return invalid(format!("local HPC pipeline-node array requires non-empty {field}"));
    }
    let mut output_ids = BTreeSet::new();
    for output in outputs {
        require_text(&output.output_id, &format!("{field}[].output_id"))?;
        require_path(&output.output_path, &format!("{field}[].output_path"))?;
        if !output_ids.insert(output.output_id.as_str()) {
            return invalid("local HPC pipeline-node output identity is duplicated");
        }
    }
    Ok(())
}

fn validate_resource_envelope(
    report: &BenchmarkLocalHpcPipelineNodeArrayReport,
) -> Result<(), BenchError> {
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
        return invalid("local HPC pipeline-node resource envelope drifted");
    }
    Ok(())
}

fn validate_resources(resources: &BenchmarkLocalHpcArrayResources) -> Result<(), BenchError> {
    if resources.cpus_per_task == 0
        || resources.memory_mb == 0
        || resources.scratch_gb == 0
        || parse_time(&resources.time_limit)? == 0
    {
        return invalid("local HPC array resources must be positive");
    }
    Ok(())
}

fn parse_time(value: &str) -> Result<u64, BenchError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 3 {
        return invalid("local HPC array time_limit must use HH:MM:SS");
    }
    let hours = parts[0].parse::<u64>().map_err(|_| {
        BenchError::InvalidPolicy("local HPC array time_limit has invalid hours".to_string())
    })?;
    let minutes = parts[1].parse::<u64>().map_err(|_| {
        BenchError::InvalidPolicy("local HPC array time_limit has invalid minutes".to_string())
    })?;
    let seconds = parts[2].parse::<u64>().map_err(|_| {
        BenchError::InvalidPolicy("local HPC array time_limit has invalid seconds".to_string())
    })?;
    if minutes >= 60 || seconds >= 60 {
        return invalid("local HPC array time_limit minute or second is out of range");
    }
    Ok(hours * 3600 + minutes * 60 + seconds)
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local HPC pipeline-node array requires non-empty {field}"));
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
            "local HPC pipeline-node array requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
