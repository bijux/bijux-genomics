//! Semantic admission for persisted local HPC candidate-run manifests.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalHpcCandidateJob, BenchmarkLocalHpcCandidateRunManifest,
    BENCHMARK_LOCAL_HPC_CANDIDATE_CPUS_PER_TASK_MAX, BENCHMARK_LOCAL_HPC_CANDIDATE_MEMORY_MB_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_PROFILE_ID, BENCHMARK_LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_CANDIDATE_SCRATCH_GB_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_STAGED_INPUT_BYTES_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_WALLTIME_MINUTES_MAX,
};

/// Validate candidate identities, resource bounds, behavior proofs, and exact aggregates.
///
/// # Errors
/// Returns an error when the selected candidate subset is incomplete or inconsistent.
pub fn validate_benchmark_local_hpc_candidate_run_manifest(
    manifest: &BenchmarkLocalHpcCandidateRunManifest,
) -> Result<(), BenchError> {
    if manifest.schema_version != BENCHMARK_LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_V1 {
        return invalid("unsupported local HPC candidate-run manifest schema");
    }
    require_repository_path(&manifest.output_path, "output_path")?;
    if manifest.selection_profile_id != BENCHMARK_LOCAL_HPC_CANDIDATE_PROFILE_ID {
        return invalid("unsupported local HPC candidate selection profile");
    }
    validate_resource_ceiling(manifest)?;
    if manifest.selected_job_count == 0
        || manifest.selected_job_count != manifest.rows.len()
        || manifest.representative_group_count != manifest.rows.len()
        || manifest.benchmark_job_count > manifest.evaluated_job_count
        || manifest.root_benchmark_job_count > manifest.benchmark_job_count
        || manifest.eligible_small_job_count > manifest.root_benchmark_job_count
        || manifest.selected_job_count > manifest.eligible_small_job_count
    {
        return invalid("local HPC candidate-run manifest aggregate does not close");
    }

    let mut groups = BTreeSet::new();
    let mut jobs = BTreeSet::new();
    let mut results = BTreeSet::new();
    let mut prior_key: Option<(&str, &str)> = None;
    for row in &manifest.rows {
        validate_row(row, manifest)?;
        if !groups.insert(row.representative_group_id.as_str())
            || !jobs.insert(row.job_id_local.as_str())
            || !results.insert(row.result_id.as_str())
        {
            return invalid("local HPC candidate-run manifest repeats a persisted identity");
        }
        let key = (row.representative_group_id.as_str(), row.result_id.as_str());
        if prior_key.is_some_and(|prior| prior >= key) {
            return invalid("local HPC candidate-run rows are not in canonical order");
        }
        prior_key = Some(key);
    }

    validate_counts(manifest)?;
    validate_exclusions(manifest)?;
    validate_behavior(manifest)?;
    if !manifest.selected_domain_counts.contains_key("fastq")
        || !manifest.selected_domain_counts.contains_key("bam")
    {
        return invalid("local HPC candidate-run manifest must cover fastq and bam");
    }
    Ok(())
}

fn validate_resource_ceiling(
    manifest: &BenchmarkLocalHpcCandidateRunManifest,
) -> Result<(), BenchError> {
    let ceiling = &manifest.resource_ceiling;
    if ceiling.cpus_per_task_max != BENCHMARK_LOCAL_HPC_CANDIDATE_CPUS_PER_TASK_MAX
        || ceiling.memory_mb_max != BENCHMARK_LOCAL_HPC_CANDIDATE_MEMORY_MB_MAX
        || ceiling.walltime_minutes_max != BENCHMARK_LOCAL_HPC_CANDIDATE_WALLTIME_MINUTES_MAX
        || ceiling.scratch_gb_max != BENCHMARK_LOCAL_HPC_CANDIDATE_SCRATCH_GB_MAX
        || ceiling.staged_input_bytes_max != BENCHMARK_LOCAL_HPC_CANDIDATE_STAGED_INPUT_BYTES_MAX
    {
        return invalid("local HPC candidate-run resource ceilings drifted from its profile");
    }
    Ok(())
}

fn validate_row(
    row: &BenchmarkLocalHpcCandidateJob,
    manifest: &BenchmarkLocalHpcCandidateRunManifest,
) -> Result<(), BenchError> {
    for (field, value) in [
        ("rows[].representative_group_id", row.representative_group_id.as_str()),
        ("rows[].selection_reason", row.selection_reason.as_str()),
        ("rows[].job_id_local", row.job_id_local.as_str()),
        ("rows[].result_id", row.result_id.as_str()),
        ("rows[].domain", row.domain.as_str()),
        ("rows[].stage_id", row.stage_id.as_str()),
        ("rows[].tool_id", row.tool_id.as_str()),
        ("rows[].corpus_id", row.corpus_id.as_str()),
        ("rows[].asset_profile_id", row.asset_profile_id.as_str()),
        ("rows[].execution_mode", row.execution_mode.as_str()),
        ("rows[].execution_mode_summary", row.execution_mode_summary.as_str()),
        ("rows[].resolution_kind", row.resolution_kind.as_str()),
        ("rows[].resolution_target", row.resolution_target.as_str()),
        (
            "rows[].expected_stage_result_manifest_path",
            row.expected_stage_result_manifest_path.as_str(),
        ),
    ] {
        require_text(value, field)?;
    }
    if !matches!(row.domain.as_str(), "fastq" | "bam" | "vcf")
        || row.stage_id.split_once('.').map(|(domain, _)| domain) != Some(row.domain.as_str())
        || row.execution_mode == "unclassified"
        || row.resolution_kind.contains("unavailable")
        || row.representative_group_id != format!("{}:{}", row.domain, row.execution_mode)
    {
        return invalid("local HPC candidate-run row identity or execution resolution is invalid");
    }
    if let Some(entrypoint) = &row.command_entrypoint {
        require_text(entrypoint, "rows[].command_entrypoint")?;
    }
    require_repository_path(
        &row.expected_stage_result_manifest_path,
        "rows[].expected_stage_result_manifest_path",
    )?;
    if row.expected_outputs.is_empty()
        || row.stop_conditions.is_empty()
        || row.staged_inputs.is_empty()
    {
        return invalid(
            "local HPC candidate-run rows require outputs, inputs, and stop conditions",
        );
    }
    if row.staged_input_count != row.staged_inputs.len()
        || row.staged_input_total_bytes
            != row.staged_inputs.iter().map(|input| input.size_bytes).sum::<u64>()
        || row.resource_hint.cpus_per_task == 0
        || row.resource_hint.memory_mb == 0
        || row.resource_hint.walltime_minutes == 0
        || row.resource_hint.cpus_per_task > manifest.resource_ceiling.cpus_per_task_max
        || row.resource_hint.memory_mb > manifest.resource_ceiling.memory_mb_max
        || row.resource_hint.walltime_minutes > manifest.resource_ceiling.walltime_minutes_max
        || row.resource_hint.scratch_gb > manifest.resource_ceiling.scratch_gb_max
        || row.staged_input_total_bytes > manifest.resource_ceiling.staged_input_bytes_max
    {
        return invalid("local HPC candidate-run row resources or staged inputs do not close");
    }
    validate_time_limit(&row.resource_hint.time_limit, row.resource_hint.walltime_minutes)?;
    let mut outputs = BTreeSet::new();
    for output in &row.expected_outputs {
        require_repository_path(output, "rows[].expected_outputs[]")?;
        if !outputs.insert(output.as_str()) {
            return invalid("local HPC candidate-run row repeats an expected output");
        }
    }
    let mut conditions = BTreeSet::new();
    for condition in &row.stop_conditions {
        require_text(&condition.condition_id, "rows[].stop_conditions[].condition_id")?;
        require_text(&condition.source_surface, "rows[].stop_conditions[].source_surface")?;
        require_text(&condition.detail, "rows[].stop_conditions[].detail")?;
        if !conditions.insert(condition.condition_id.as_str()) {
            return invalid("local HPC candidate-run row repeats a stop condition");
        }
    }
    if row.staged_inputs.iter().any(|input| input.consumer_result_id != row.result_id) {
        return invalid("local HPC candidate-run staged-input consumer lineage drifted");
    }
    Ok(())
}

fn validate_counts(manifest: &BenchmarkLocalHpcCandidateRunManifest) -> Result<(), BenchError> {
    if manifest.selected_staged_input_count
        != manifest.rows.iter().map(|row| row.staged_input_count).sum::<usize>()
        || manifest.selected_staged_input_bytes
            != manifest.rows.iter().map(|row| row.staged_input_total_bytes).sum::<u64>()
        || manifest.selected_domain_counts != count_by(&manifest.rows, |row| &row.domain)
        || manifest.selected_execution_mode_counts
            != count_by(&manifest.rows, |row| &row.execution_mode)
        || manifest.selected_resolution_kind_counts
            != count_by(&manifest.rows, |row| &row.resolution_kind)
    {
        return invalid("local HPC candidate-run derived counters do not close");
    }
    Ok(())
}

fn validate_exclusions(manifest: &BenchmarkLocalHpcCandidateRunManifest) -> Result<(), BenchError> {
    if manifest
        .exclusion_reason_counts
        .iter()
        .any(|(reason, count)| reason.trim().is_empty() || *count == 0)
        || manifest.exclusion_reason_counts.values().sum::<usize>() + manifest.selected_job_count
            != manifest.evaluated_job_count
        || manifest.exclusion_reason_counts.get("non_benchmark_result").copied().unwrap_or_default()
            != manifest.evaluated_job_count - manifest.benchmark_job_count
        || manifest.exclusion_reason_counts.get("has_dependencies").copied().unwrap_or_default()
            != manifest.benchmark_job_count - manifest.root_benchmark_job_count
        || manifest
            .exclusion_reason_counts
            .get("superseded_by_smaller_representative")
            .copied()
            .unwrap_or_default()
            != manifest.eligible_small_job_count - manifest.selected_job_count
    {
        return invalid("local HPC candidate-run exclusion accounting does not close");
    }
    Ok(())
}

fn validate_behavior(manifest: &BenchmarkLocalHpcCandidateRunManifest) -> Result<(), BenchError> {
    let behavior = &manifest.behavior;
    let flags = [
        behavior.only_benchmark_result_jobs,
        behavior.only_dependency_free_jobs,
        behavior.only_known_assets,
        behavior.only_known_execution_modes,
        behavior.only_available_execution_resolution,
        behavior.only_small_resource_envelopes,
        behavior.only_small_input_footprints,
        behavior.selected_rows_keep_expected_outputs,
        behavior.selected_rows_keep_stop_conditions,
        behavior.vcf_rows_excluded_for_unknown_execution_modes,
    ];
    if !behavior.proven || flags.iter().any(|flag| !flag) {
        return invalid("local HPC candidate-run behavior proof is incomplete");
    }
    Ok(())
}

fn count_by<'a>(
    rows: &'a [BenchmarkLocalHpcCandidateJob],
    key: impl Fn(&'a BenchmarkLocalHpcCandidateJob) -> &'a str,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for row in rows {
        *counts.entry(key(row).to_string()).or_default() += 1;
    }
    counts
}

fn validate_time_limit(value: &str, expected_minutes: u32) -> Result<(), BenchError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.len() != 2 || !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return invalid("local HPC candidate-run time limit must use HH:MM:SS");
    }
    let hours = parts[0]
        .parse::<u32>()
        .map_err(|_| BenchError::InvalidPolicy("invalid candidate hours".into()))?;
    let minutes = parts[1]
        .parse::<u32>()
        .map_err(|_| BenchError::InvalidPolicy("invalid candidate minutes".into()))?;
    let seconds = parts[2]
        .parse::<u32>()
        .map_err(|_| BenchError::InvalidPolicy("invalid candidate seconds".into()))?;
    if minutes >= 60 || seconds != 0 || hours * 60 + minutes != expected_minutes {
        return invalid("local HPC candidate-run walltime does not match time_limit");
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local HPC candidate-run manifest requires non-empty {field}"));
    }
    Ok(())
}

fn require_repository_path(value: &str, field: &str) -> Result<(), BenchError> {
    require_text(value, field)?;
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!(
            "local HPC candidate-run manifest requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
