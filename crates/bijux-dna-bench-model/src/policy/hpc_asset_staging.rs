//! Owner: canonical staging assembly and current-input comparison.
use crate::{
    validate_benchmark_local_hpc_asset_staging_manifest, BenchmarkLocalHpcAssetStagingJob as Job,
    BenchmarkLocalHpcAssetStagingManifest as Manifest, BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1,
};
use anyhow::{anyhow, Result};
use std::path::Path;
/// Assemble and admit the canonical staging manifest from observed job inputs.
/// # Errors
/// Returns identity, ordering, lineage or content-summary failures.
pub fn assemble_hpc_asset_staging_manifest(
    output_path: String,
    staging_root: String,
    mut jobs: Vec<Job>,
) -> Result<Manifest> {
    jobs.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });
    let source_paths = jobs
        .iter()
        .flat_map(|job| job.staged_inputs.iter().map(|input| input.source_path.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    let manifest = Manifest {
        schema_version: BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1.to_string(),
        output_path,
        staging_root,
        selected_job_count: jobs.len(),
        staged_input_count: jobs.iter().map(|job| job.staged_inputs.len()).sum(),
        unique_source_path_count: source_paths.len(),
        jobs,
    };
    validate_benchmark_local_hpc_asset_staging_manifest(&manifest)?;
    Ok(manifest)
}
/// Compare staging evidence with current governed inputs.
/// # Errors
/// Returns field-specific drift diagnostics.
pub fn compare_hpc_asset_staging_manifest(
    manifest_path: &Path,
    observed: &Manifest,
    expected: &Manifest,
) -> Result<()> {
    if observed.output_path != expected.output_path {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in output_path: observed `{}` expected `{}`",
            manifest_path.display(),
            observed.output_path,
            expected.output_path
        ));
    }
    if observed.staging_root != expected.staging_root {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in staging_root: observed `{}` expected `{}`",
            manifest_path.display(),
            observed.staging_root,
            expected.staging_root
        ));
    }
    if observed.selected_job_count != expected.selected_job_count {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in selected_job_count: observed {} expected {}",
            manifest_path.display(),
            observed.selected_job_count,
            expected.selected_job_count
        ));
    }
    if observed.staged_input_count != expected.staged_input_count {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in staged_input_count: observed {} expected {}",
            manifest_path.display(),
            observed.staged_input_count,
            expected.staged_input_count
        ));
    }
    if observed.unique_source_path_count != expected.unique_source_path_count {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in unique_source_path_count: observed {} expected {}",
            manifest_path.display(),
            observed.unique_source_path_count,
            expected.unique_source_path_count
        ));
    }
    if observed.jobs.len() != expected.jobs.len() {
        return Err(anyhow!(
            "HPC asset staging manifest `{}` drifted in job count: observed {} expected {}",
            manifest_path.display(),
            observed.jobs.len(),
            expected.jobs.len()
        ));
    }

    for (index, (observed_job, expected_job)) in
        observed.jobs.iter().zip(expected.jobs.iter()).enumerate()
    {
        compare_job(manifest_path, index, observed_job, expected_job)?;
    }

    Ok(())
}

fn compare_job(
    manifest_path: &Path,
    index: usize,
    observed_job: &Job,
    expected_job: &Job,
) -> Result<()> {
    if observed_job.result_id != expected_job.result_id
        || observed_job.domain != expected_job.domain
        || observed_job.stage_id != expected_job.stage_id
        || observed_job.tool_id != expected_job.tool_id
        || observed_job.corpus_id != expected_job.corpus_id
        || observed_job.asset_profile_id != expected_job.asset_profile_id
    {
        return Err(anyhow!(
                "HPC asset staging manifest `{}` drifted in job {}: observed `{}` / `{}` / `{}` / `{}` / `{}` / `{}` expected `{}` / `{}` / `{}` / `{}` / `{}` / `{}`",
                manifest_path.display(),
                index,
                observed_job.result_id,
                observed_job.domain,
                observed_job.stage_id,
                observed_job.tool_id,
                observed_job.corpus_id,
                observed_job.asset_profile_id,
                expected_job.result_id,
                expected_job.domain,
                expected_job.stage_id,
                expected_job.tool_id,
                expected_job.corpus_id,
                expected_job.asset_profile_id
            ));
    }
    if observed_job.staged_inputs.len() != expected_job.staged_inputs.len() {
        return Err(anyhow!(
                "HPC asset staging manifest `{}` drifted in staged input count for `{}`: observed {} expected {}",
                manifest_path.display(),
                observed_job.result_id,
                observed_job.staged_inputs.len(),
                expected_job.staged_inputs.len()
            ));
    }
    for (input_index, (observed_input, expected_input)) in
        observed_job.staged_inputs.iter().zip(expected_job.staged_inputs.iter()).enumerate()
    {
        if observed_input != expected_input {
            return Err(anyhow!(
                    "HPC asset staging manifest `{}` drifted in staged input {} for `{}` at source `{}`",
                    manifest_path.display(),
                    input_index,
                    observed_job.result_id,
                    expected_input.source_path
                ));
        }
    }
    Ok(())
}
