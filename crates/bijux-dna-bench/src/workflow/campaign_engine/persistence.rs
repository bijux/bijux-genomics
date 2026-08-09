//! Owner: bijux-dna-bench
//! Immutable persistence and replay validation for generic campaign evidence closure.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use super::{
    aggregate_benchmark_case_observations, validate_benchmark_case_aggregations,
    validate_benchmark_case_matrix, BenchmarkAdmittedCaseObservationSetV1,
    BenchmarkAdmittedCaseObservationV2, BenchmarkCaseAggregationSetV1, BenchmarkCaseMatrixV2,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1, BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1,
};

const MAX_CAMPAIGN_EVIDENCE_BYTES: usize = 64 * 1024 * 1024;

/// Return the deterministic compact-JSON identity of a valid case matrix.
///
/// # Errors
/// Returns an error when the matrix is invalid or cannot be serialized.
pub fn benchmark_case_matrix_sha256(matrix: &BenchmarkCaseMatrixV2) -> Result<String> {
    validate_benchmark_case_matrix(matrix)?;
    hash_json(matrix)
}

/// Canonicalize and validate complete admitted observation closure for a case matrix.
///
/// # Errors
/// Returns an error when observations are duplicated, incomplete, invalid, or source-drifted.
pub fn build_benchmark_admitted_case_observation_set(
    matrix: &BenchmarkCaseMatrixV2,
    observations: Vec<BenchmarkAdmittedCaseObservationV2>,
) -> Result<BenchmarkAdmittedCaseObservationSetV1> {
    let matrix_sha256 = benchmark_case_matrix_sha256(matrix)?;
    aggregate_benchmark_case_observations(matrix, &observations)
        .context("validate admitted benchmark observation closure")?;
    let mut indexed = observations
        .into_iter()
        .map(|observation| (observation.case_id.clone(), observation))
        .collect::<BTreeMap<_, _>>();
    let observations = matrix
        .cases
        .iter()
        .map(|case| {
            indexed.remove(&case.case_id).context("validated observation closure lost a case")
        })
        .collect::<Result<Vec<_>>>()?;
    if !indexed.is_empty() {
        bail!("validated observation closure retained undeclared cases");
    }
    Ok(BenchmarkAdmittedCaseObservationSetV1 {
        schema_version: BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1.to_string(),
        repository_commit: matrix.repository_commit.clone(),
        case_matrix_sha256: matrix_sha256,
        observations,
    })
}

/// Validate persisted observation closure by exact canonical replay.
///
/// # Errors
/// Returns an error when schema, source identity, matrix identity, case order, or evidence differs.
pub fn validate_benchmark_admitted_case_observation_set(
    matrix: &BenchmarkCaseMatrixV2,
    set: &BenchmarkAdmittedCaseObservationSetV1,
) -> Result<()> {
    if set.schema_version != BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1
        || set.repository_commit != matrix.repository_commit
        || set.case_matrix_sha256 != benchmark_case_matrix_sha256(matrix)?
    {
        bail!("admitted benchmark observation set authority differs from its case matrix");
    }
    let replay = build_benchmark_admitted_case_observation_set(matrix, set.observations.clone())?;
    if replay != *set {
        bail!("admitted benchmark observation set differs from canonical replay");
    }
    Ok(())
}

/// Derive source-bound aggregation closure from a validated observation set.
///
/// # Errors
/// Returns an error when observation closure or aggregation derivation is invalid.
pub fn aggregate_benchmark_admitted_case_observation_set(
    matrix: &BenchmarkCaseMatrixV2,
    observations: &BenchmarkAdmittedCaseObservationSetV1,
) -> Result<BenchmarkCaseAggregationSetV1> {
    validate_benchmark_admitted_case_observation_set(matrix, observations)?;
    let aggregations = aggregate_benchmark_case_observations(matrix, &observations.observations)?;
    Ok(BenchmarkCaseAggregationSetV1 {
        schema_version: BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1.to_string(),
        repository_commit: matrix.repository_commit.clone(),
        case_matrix_sha256: observations.case_matrix_sha256.clone(),
        observation_set_sha256: hash_json(observations)?,
        aggregations,
    })
}

/// Validate persisted aggregation closure against both upstream authorities.
///
/// # Errors
/// Returns an error when schema, source, digest, aggregation, or replay closure differs.
pub fn validate_benchmark_case_aggregation_set(
    matrix: &BenchmarkCaseMatrixV2,
    observations: &BenchmarkAdmittedCaseObservationSetV1,
    aggregations: &BenchmarkCaseAggregationSetV1,
) -> Result<()> {
    validate_benchmark_admitted_case_observation_set(matrix, observations)?;
    if aggregations.schema_version != BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1
        || aggregations.repository_commit != matrix.repository_commit
        || aggregations.case_matrix_sha256 != benchmark_case_matrix_sha256(matrix)?
        || aggregations.observation_set_sha256 != hash_json(observations)?
    {
        bail!("benchmark aggregation set authority differs from its evidence closure");
    }
    validate_benchmark_case_aggregations(matrix, &aggregations.aggregations)?;
    if aggregate_benchmark_admitted_case_observation_set(matrix, observations)? != *aggregations {
        bail!("benchmark aggregation set differs from canonical replay");
    }
    Ok(())
}

/// No-clobber persist a validated case matrix.
///
/// # Errors
/// Returns an error when validation, existing-content comparison, or atomic creation fails.
pub fn ensure_benchmark_case_matrix(path: &Path, matrix: &BenchmarkCaseMatrixV2) -> Result<()> {
    validate_benchmark_case_matrix(matrix)?;
    ensure_json_identity(path, matrix, "benchmark case matrix")
}

/// Load and validate a bounded case matrix file.
///
/// # Errors
/// Returns an error when the file is unsafe, oversized, malformed, or invalid.
pub fn load_benchmark_case_matrix(path: &Path) -> Result<BenchmarkCaseMatrixV2> {
    let matrix = read_json(path, "benchmark case matrix")?;
    validate_benchmark_case_matrix(&matrix)?;
    Ok(matrix)
}

/// No-clobber persist a validated admitted observation set.
///
/// # Errors
/// Returns an error when validation, existing-content comparison, or atomic creation fails.
pub fn ensure_benchmark_admitted_case_observation_set(
    path: &Path,
    matrix: &BenchmarkCaseMatrixV2,
    set: &BenchmarkAdmittedCaseObservationSetV1,
) -> Result<()> {
    validate_benchmark_admitted_case_observation_set(matrix, set)?;
    ensure_json_identity(path, set, "admitted benchmark observation set")
}

/// Load and validate a bounded admitted observation set file.
///
/// # Errors
/// Returns an error when the file is unsafe, oversized, malformed, or source-drifted.
pub fn load_benchmark_admitted_case_observation_set(
    path: &Path,
    matrix: &BenchmarkCaseMatrixV2,
) -> Result<BenchmarkAdmittedCaseObservationSetV1> {
    let set = read_json(path, "admitted benchmark observation set")?;
    validate_benchmark_admitted_case_observation_set(matrix, &set)?;
    Ok(set)
}

/// No-clobber persist a validated aggregation set.
///
/// # Errors
/// Returns an error when validation, existing-content comparison, or atomic creation fails.
pub fn ensure_benchmark_case_aggregation_set(
    path: &Path,
    matrix: &BenchmarkCaseMatrixV2,
    observations: &BenchmarkAdmittedCaseObservationSetV1,
    aggregations: &BenchmarkCaseAggregationSetV1,
) -> Result<()> {
    validate_benchmark_case_aggregation_set(matrix, observations, aggregations)?;
    ensure_json_identity(path, aggregations, "benchmark aggregation set")
}

/// Load and validate a bounded aggregation set file.
///
/// # Errors
/// Returns an error when the file is unsafe, oversized, malformed, or source-drifted.
pub fn load_benchmark_case_aggregation_set(
    path: &Path,
    matrix: &BenchmarkCaseMatrixV2,
    observations: &BenchmarkAdmittedCaseObservationSetV1,
) -> Result<BenchmarkCaseAggregationSetV1> {
    let set = read_json(path, "benchmark aggregation set")?;
    validate_benchmark_case_aggregation_set(matrix, observations, &set)?;
    Ok(set)
}

fn hash_json(value: &impl Serialize) -> Result<String> {
    Ok(bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(value)?))
}

fn ensure_json_identity<T>(path: &Path, value: &T, label: &str) -> Result<()>
where
    T: Serialize + DeserializeOwned + PartialEq,
{
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("existing {label} path is not a regular file");
            }
            let existing: T = read_json(path, label)?;
            if existing != *value {
                bail!("existing {label} conflicts with immutable evidence");
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(path, value)
                .with_context(|| format!("persist {label} {}", path.display()))
        }
        Err(error) => Err(error).with_context(|| format!("inspect {label} {}", path.display())),
    }
}

fn read_json<T: DeserializeOwned>(path: &Path, label: &str) -> Result<T> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} path is not a regular file");
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_CAMPAIGN_EVIDENCE_BYTES)
        .with_context(|| format!("read {label} {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {label} {}", path.display()))
}
