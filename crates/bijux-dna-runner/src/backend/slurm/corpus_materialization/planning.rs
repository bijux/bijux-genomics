//! Owner: bijux-dna-runner
//! Source-locked planning for governed corpus materialization scheduler specifications.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{validate_corpus_acquisition_plan, CorpusAcquisitionPlan};

use super::contracts::{
    CorpusMaterializationSlurmPlanRequest, CorpusMaterializationSlurmSpec,
    CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V2,
};
use super::validation::{
    canonical_directory, catalog_snapshot_path, ensure_json_identity, read_json,
    validate_catalog_closure, validate_catalog_precondition, validate_specification,
};

/// Derive and persist one immutable corpus-materialization scheduler specification.
///
/// Source commit, executable and plan digests, corpus and portfolio identities, catalog
/// precondition, command projection, and the governed evidence root are derived rather than
/// accepted as caller claims.
///
/// # Errors
/// Returns an error when the repository is dirty, an authority is unsafe or inconsistent, the
/// acquisition plan is invalid, resources are invalid, or immutable evidence conflicts.
pub fn plan_corpus_materialization_slurm_spec(
    request: &CorpusMaterializationSlurmPlanRequest,
) -> Result<CorpusMaterializationSlurmSpec> {
    let repository_path = request
        .repository_path
        .canonicalize()
        .context("canonicalize corpus materialization repository")?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&repository_path)
            .context("require a clean corpus materialization repository")?;
    let executable_path = request
        .executable_path
        .canonicalize()
        .context("canonicalize corpus materialization executable")?;
    let data_root = request.data_root.canonicalize().context("canonicalize governed data root")?;
    let plan_path = request.plan_path.canonicalize().context("canonicalize acquisition plan")?;
    plan_path.strip_prefix(&data_root).context("acquisition plan escapes governed data root")?;
    let plan: CorpusAcquisitionPlan = read_json(&plan_path)?;
    validate_corpus_acquisition_plan(&plan).context("validate corpus acquisition plan")?;

    let evidence_root = data_root
        .join("evidence/materializations")
        .join(&repository_commit)
        .join(&plan.corpus_id)
        .join(&request.attempt_id);
    let mut spec = CorpusMaterializationSlurmSpec {
        schema_version: CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V2.to_string(),
        run_id: request.run_id.clone(),
        attempt_id: request.attempt_id.clone(),
        corpus_id: plan.corpus_id.clone(),
        portfolio_id: plan.portfolio_id.clone(),
        repository_path,
        repository_commit,
        executable_sha256: bijux_dna_infra::hash_file_sha256(&executable_path)
            .context("hash corpus materialization executable")?,
        executable_path,
        command_arguments: Vec::new(),
        plan_sha256: bijux_dna_infra::hash_file_sha256(&plan_path)
            .context("hash corpus acquisition plan")?,
        plan_path,
        existing_catalog_sha256: None,
        data_root,
        evidence_root,
        resources: request.resources.clone(),
        maximum_parallelism: request.maximum_parallelism,
        retries: request.retries,
        timeout_seconds: request.timeout_seconds,
        admitted_at_utc: request.admitted_at_utc.clone(),
    };
    spec.command_arguments = materialization_command_arguments(&spec);
    spec.existing_catalog_sha256 = existing_catalog_identity(&spec)?;
    validate_specification(&spec)?;
    validate_catalog_precondition(&spec)?;
    if let Some(expected_sha256) = &spec.existing_catalog_sha256 {
        let (_, observed_sha256) = validate_catalog_closure(&spec, &plan)?;
        if observed_sha256 != *expected_sha256 {
            bail!("existing governed data catalog closure changed during planning");
        }
    }
    let persisted_root = ensure_planning_evidence_root(
        &spec.data_root,
        &spec.repository_commit,
        &spec.corpus_id,
        &spec.attempt_id,
    )?;
    if persisted_root != spec.evidence_root {
        bail!("derived corpus materialization evidence root is not canonical");
    }
    ensure_json_identity(&spec.evidence_root.join("materialization-spec.json"), &spec)?;
    Ok(spec)
}

fn materialization_command_arguments(spec: &CorpusMaterializationSlurmSpec) -> Vec<String> {
    vec![
        "bench".to_string(),
        "materialize-fastq-corpus".to_string(),
        "--plan".to_string(),
        spec.plan_path.display().to_string(),
        "--data-root".to_string(),
        spec.data_root.display().to_string(),
        "--repository-commit".to_string(),
        spec.repository_commit.clone(),
        "--jobs".to_string(),
        spec.maximum_parallelism.to_string(),
        "--retries".to_string(),
        spec.retries.to_string(),
        "--timeout-seconds".to_string(),
        spec.timeout_seconds.to_string(),
        "--admitted-at-utc".to_string(),
        spec.admitted_at_utc.clone(),
    ]
}

fn existing_catalog_identity(spec: &CorpusMaterializationSlurmSpec) -> Result<Option<String>> {
    let path = catalog_snapshot_path(&spec.data_root, spec);
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("existing governed data catalog is not a regular non-symlink file");
            }
            if path.canonicalize().context("canonicalize existing governed data catalog")? != path {
                bail!("existing governed data catalog path is not canonical");
            }
            Ok(Some(
                bijux_dna_infra::hash_file_sha256(&path)
                    .context("hash existing governed data catalog")?,
            ))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

fn ensure_planning_evidence_root(
    data_root: &Path,
    repository_commit: &str,
    corpus_id: &str,
    attempt_id: &str,
) -> Result<PathBuf> {
    let mut current = canonical_directory(
        &data_root.join("evidence/materializations"),
        "corpus materialization evidence authority",
    )?;
    for component in [repository_commit, corpus_id, attempt_id] {
        current = ensure_directory(&current.join(component))?;
    }
    Ok(current)
}

fn ensure_directory(path: &Path) -> Result<PathBuf> {
    match fs::create_dir(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error).with_context(|| format!("create {}", path.display())),
    }
    canonical_directory(path, "corpus materialization evidence directory")
}
