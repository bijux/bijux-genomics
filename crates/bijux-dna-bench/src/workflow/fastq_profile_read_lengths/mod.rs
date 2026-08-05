//! Owner: bijux-dna-bench
//! Executable material for the governed FASTQ read-length campaign.

mod commands;
mod inputs;
mod parameters;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric,
    BenchmarkAcceptanceOperator, BenchmarkAttemptIdentity, BenchmarkExecutionAcceptance,
    BenchmarkExecutionDeterminism, BenchmarkLifecycleStatus,
};

use self::commands::{measured_command, negative_command, required_files, warmup_command};
use self::inputs::execution_inputs;
use self::parameters::ensure_parameter_evidence;
use super::{
    ensure_fastq_read_length_expectations, load_attempt_lifecycle, BenchmarkExecutionMaterial,
    FastqReadLengthExpectationPublication,
};

const RUN_ID: &str = "run-2042";
const STAGE_ID: &str = "fastq.profile_read_lengths";
const CORPUS_ID: &str = "human-short-read-diversity";

#[derive(Debug, Clone, PartialEq)]
pub struct FastqProfileReadLengthsMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub expectations: FastqReadLengthExpectationPublication,
}

#[allow(clippy::too_many_arguments)]
/// Prepare complete executable material for one planned run-2042 tool attempt.
///
/// # Errors
/// Returns an error when attempt identity/state, source locks, corpus evidence, parameter
/// authority, input mounts, commands, or acceptance closure differs from the governed campaign.
pub fn prepare_fastq_profile_read_lengths_material(
    repository_root: &Path,
    study_root: &Path,
    data_index_path: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
) -> Result<FastqProfileReadLengthsMaterial> {
    validate_attempt_identity(identity).context("validate FASTQ attempt identity")?;
    if identity.run.run_id != RUN_ID
        || identity.run.domain != "fastq"
        || identity.run.stage_id != STAGE_ID
        || !matches!(identity.tool_id.as_str(), "seqfu" | "seqkit")
    {
        bail!("attempt identity is outside the governed run-2042 closure");
    }
    let lifecycle = load_attempt_lifecycle(benchmark_root, identity)?;
    if lifecycle.current_status() != Some(BenchmarkLifecycleStatus::Planned) {
        bail!("FASTQ execution material requires a wholly planned attempt");
    }
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let expectations =
        ensure_fastq_read_length_expectations(study_root, data_index_path, CORPUS_ID, 20, 30)?;
    let parameters = ensure_parameter_evidence(
        repository_root,
        &attempt_directory,
        &expectations.admitted_corpus,
    )?;
    let (inputs, samples) = execution_inputs(&expectations)?;
    let measured_repetitions = 3;
    let execution = BenchmarkExecutionMaterial {
        operator: operator.to_string(),
        prepared_at_utc: prepared_at_utc.to_string(),
        repository_path: PathBuf::from(repository_root),
        benchmark_root: PathBuf::from(benchmark_root),
        source_contract_sha256: source_contract_sha256.clone(),
        inputs,
        warmup_command: warmup_command(&identity.tool_id, &samples),
        measured_command: measured_command(
            &identity.tool_id,
            &samples,
            expectations.admitted_corpus.input_bytes,
        ),
        negative_command: negative_command(&identity.tool_id),
        parameters,
        required_files: required_files(&samples, measured_repetitions),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: 8192,
        walltime: "01:00:00".to_string(),
        scratch_mb: 4096,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
    };
    Ok(FastqProfileReadLengthsMaterial { execution, expectations })
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: String::new(),
        negative_exit_nonzero: true,
        metrics: vec![
            metric("sample_count", BenchmarkAcceptanceOperator::Equal, 20, "samples"),
            metric("file_count", BenchmarkAcceptanceOperator::Equal, 30, "files"),
            metric("read_count", BenchmarkAcceptanceOperator::GreaterThan, 0, "reads"),
            metric(
                "exact_observation_mismatch_count",
                BenchmarkAcceptanceOperator::Equal,
                0,
                "samples",
            ),
            metric("histogram_mismatch_count", BenchmarkAcceptanceOperator::Equal, 0, "samples"),
        ],
    }
}

fn metric(
    id: &str,
    operator: BenchmarkAcceptanceOperator,
    expected: i64,
    unit: &str,
) -> BenchmarkAcceptanceMetric {
    BenchmarkAcceptanceMetric {
        id: id.to_string(),
        path: "repetitions/repetition-1.metrics.json".to_string(),
        json_pointer: format!("/{id}"),
        operator,
        expected: BenchmarkAcceptanceExpectedValue::Integer(expected),
        unit: unit.to_string(),
    }
}
