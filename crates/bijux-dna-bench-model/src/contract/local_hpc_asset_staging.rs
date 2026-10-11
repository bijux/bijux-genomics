//! Validation for persisted local HPC asset-staging manifests.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalHpcAssetStagingInput, BenchmarkLocalHpcAssetStagingJob,
    BenchmarkLocalHpcAssetStagingManifest, BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1,
};

#[derive(Clone, Copy, PartialEq, Eq)]
struct SourceIdentity<'a> {
    source_kind: &'a str,
    staged_path: &'a str,
    checksum_sha256: &'a str,
    size_bytes: u64,
    member_count: usize,
}

/// Validate staging identity, containment, ordering, checksums, and exact counters.
///
/// # Errors
/// Returns an error when persisted asset-staging evidence is incomplete or inconsistent.
pub fn validate_benchmark_local_hpc_asset_staging_manifest(
    manifest: &BenchmarkLocalHpcAssetStagingManifest,
) -> Result<(), BenchError> {
    if manifest.schema_version != BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1 {
        return invalid(format!(
            "unsupported local HPC asset-staging schema `{}`",
            manifest.schema_version
        ));
    }
    require_repository_path(&manifest.output_path, "output_path")?;
    require_repository_path(&manifest.staging_root, "staging_root")?;
    if Path::new(&manifest.output_path).starts_with(&manifest.staging_root)
        || Path::new(&manifest.staging_root).starts_with(&manifest.output_path)
    {
        return invalid("local HPC asset-staging manifest and staging root must not overlap");
    }
    if manifest.selected_job_count != manifest.jobs.len() {
        return invalid("selected_job_count aligned with jobs invariant failed");
    }
    let staged_input_count = manifest.jobs.iter().map(|job| job.staged_inputs.len()).sum::<usize>();
    if manifest.staged_input_count != staged_input_count {
        return invalid("staged_input_count aligned with staged inputs invariant failed");
    }

    let mut result_ids = BTreeSet::new();
    let mut source_identities = BTreeMap::<&str, SourceIdentity<'_>>::new();
    let mut prior_job_key: Option<(&str, &str, &str, &str)> = None;
    for job in &manifest.jobs {
        validate_job(job, &manifest.staging_root, &mut source_identities)?;
        if !result_ids.insert(job.result_id.as_str()) {
            return invalid(format!("duplicate asset-staging result `{}`", job.result_id));
        }
        let job_key = (
            job.domain.as_str(),
            job.stage_id.as_str(),
            job.tool_id.as_str(),
            job.result_id.as_str(),
        );
        if prior_job_key.is_some_and(|prior| prior >= job_key) {
            return invalid("asset-staging jobs are not in canonical order");
        }
        prior_job_key = Some(job_key);
    }
    if manifest.unique_source_path_count != source_identities.len() {
        return invalid("unique_source_path_count aligned with source paths invariant failed");
    }
    Ok(())
}

fn validate_job<'a>(
    job: &'a BenchmarkLocalHpcAssetStagingJob,
    staging_root: &str,
    source_identities: &mut BTreeMap<&'a str, SourceIdentity<'a>>,
) -> Result<(), BenchError> {
    for (value, field) in [
        (&job.result_id, "jobs[].result_id"),
        (&job.domain, "jobs[].domain"),
        (&job.stage_id, "jobs[].stage_id"),
        (&job.tool_id, "jobs[].tool_id"),
        (&job.corpus_id, "jobs[].corpus_id"),
        (&job.asset_profile_id, "jobs[].asset_profile_id"),
    ] {
        require_text(value, field)?;
    }
    if job.staged_inputs.is_empty() {
        return invalid(format!("asset-staging job `{}` has no inputs", job.result_id));
    }
    let mut prior_source: Option<&str> = None;
    for input in &job.staged_inputs {
        validate_input(input, job, staging_root)?;
        if prior_source.is_some_and(|prior| prior >= input.source_path.as_str()) {
            return invalid(format!(
                "asset-staging job `{}` inputs are not in canonical order",
                job.result_id
            ));
        }
        prior_source = Some(input.source_path.as_str());
        let identity = SourceIdentity {
            source_kind: input.source_kind.as_str(),
            staged_path: input.staged_path.as_str(),
            checksum_sha256: input.checksum_sha256.as_str(),
            size_bytes: input.size_bytes,
            member_count: input.member_count,
        };
        if source_identities
            .insert(input.source_path.as_str(), identity)
            .is_some_and(|prior| prior != identity)
        {
            return invalid(format!(
                "asset-staging source `{}` has inconsistent identities",
                input.source_path
            ));
        }
    }
    Ok(())
}

fn validate_input(
    input: &BenchmarkLocalHpcAssetStagingInput,
    job: &BenchmarkLocalHpcAssetStagingJob,
    staging_root: &str,
) -> Result<(), BenchError> {
    require_text(&input.source_kind, "jobs[].staged_inputs[].source_kind")?;
    require_repository_path(&input.source_path, "jobs[].staged_inputs[].source_path")?;
    require_repository_path(&input.staged_path, "jobs[].staged_inputs[].staged_path")?;
    validate_optional_text(input.artifact_id.as_deref(), "jobs[].staged_inputs[].artifact_id")?;
    validate_optional_text(input.artifact_role.as_deref(), "jobs[].staged_inputs[].artifact_role")?;
    if !matches!(input.source_kind.as_str(), "file" | "directory" | "prefix_bundle") {
        return invalid(format!(
            "asset-staging job `{}` has unsupported source kind `{}`",
            job.result_id, input.source_kind
        ));
    }
    if input.artifact_id.is_some() != input.artifact_role.is_some() {
        return invalid(format!(
            "asset-staging job `{}` has incomplete artifact identity",
            job.result_id
        ));
    }
    if input.consumer_result_id != job.result_id {
        return invalid(format!(
            "asset-staging input `{}` drifted from consumer `{}`",
            input.source_path, job.result_id
        ));
    }
    if Path::new(&input.source_path).starts_with(staging_root) {
        return invalid(format!(
            "asset-staging input `{}` is already inside the staging root",
            input.source_path
        ));
    }
    let expected_staged_path = Path::new(staging_root).join(&input.source_path);
    if Path::new(&input.staged_path) != expected_staged_path {
        return invalid(format!(
            "asset-staging input `{}` has invalid staged lineage",
            input.source_path
        ));
    }
    if input.checksum_sha256.len() != 64
        || !input.checksum_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return invalid(format!(
            "asset-staging input `{}` has an invalid checksum",
            input.source_path
        ));
    }
    if input.member_count == 0 {
        return invalid(format!("asset-staging input `{}` is empty", input.source_path));
    }
    Ok(())
}

fn validate_optional_text(value: Option<&str>, field: &str) -> Result<(), BenchError> {
    if let Some(value) = value {
        require_text(value, field)?;
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local HPC asset staging requires non-empty {field}"));
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
            "local HPC asset staging requires repository-relative {field} without traversal"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
