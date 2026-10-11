//! Owner: bijux-dna-bench
//! Executable material for the governed FASTQ adapter-inspection campaign.

mod commands;
mod parameters;
mod report;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric,
    BenchmarkAcceptanceOperator, BenchmarkAttemptIdentity, BenchmarkExecutionAcceptance,
    BenchmarkExecutionDeterminism, BenchmarkLifecycleStatus,
};

use self::commands::{measured_command, negative_command, required_files, warmup_command};
use self::parameters::ensure_parameter_evidence;
use super::{
    fastq_profile_reads::campaign_execution_inputs, load_attempt_lifecycle,
    BenchmarkExecutionMaterial, FastqProfileReadsExpectationPublication,
};

pub use report::{
    publish_fastq_detect_adapters_stage_report, render_fastq_detect_adapters_markdown,
    FastqDetectAdaptersStageReportPublication,
};

const RUN_ID: &str = "run-2030";
const STAGE_ID: &str = "fastq.detect_adapters";
const CORPUS_ID: &str = "human-short-read-diversity";

#[derive(Debug, Clone, PartialEq)]
pub struct FastqDetectAdaptersMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub expectations: FastqProfileReadsExpectationPublication,
}

#[allow(clippy::too_many_arguments)]
/// Prepare one planned run-2030 tool attempt from campaign-wide immutable expectations.
///
/// # Errors
/// Returns an error when attempt state, corpus truth, parameters, commands, or acceptance closure
/// differs from the governed adapter-inspection campaign.
pub fn prepare_fastq_detect_adapters_material_from_expectations(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    expectations: &FastqProfileReadsExpectationPublication,
) -> Result<FastqDetectAdaptersMaterial> {
    validate_attempt_identity(identity)
        .context("validate FASTQ detect-adapters attempt identity")?;
    if identity.run.run_id != RUN_ID
        || identity.run.domain != "fastq"
        || identity.run.stage_id != STAGE_ID
        || identity.tool_id != "fastqc"
    {
        bail!("attempt identity is outside the governed run-2030 closure");
    }
    let lifecycle = load_attempt_lifecycle(benchmark_root, identity)?;
    if lifecycle.current_status() != Some(BenchmarkLifecycleStatus::Planned) {
        bail!("FASTQ detect-adapters material requires a wholly planned attempt");
    }
    if expectations.admitted_corpus.corpus_id != CORPUS_ID
        || expectations.admitted_corpus.sample_count != 20
        || expectations.admitted_corpus.file_count != 30
        || expectations.samples.len() != 20
    {
        bail!("FASTQ detect-adapters expectation closure differs from the campaign");
    }
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let (inputs, samples) = campaign_execution_inputs(expectations)?;
    let parameters = ensure_parameter_evidence(repository_root, &attempt_directory, &samples)?;
    let measured_repetitions = 3;
    let execution = BenchmarkExecutionMaterial {
        operator: operator.to_string(),
        prepared_at_utc: prepared_at_utc.to_string(),
        repository_path: PathBuf::from(repository_root),
        benchmark_root: PathBuf::from(benchmark_root),
        source_contract_sha256: source_contract_sha256.clone(),
        inputs,
        warmup_command: warmup_command(&identity.tool_id, &samples)?,
        measured_command: measured_command(
            &identity.tool_id,
            &samples,
            expectations.admitted_corpus.input_bytes,
        )?,
        negative_command: negative_command(&identity.tool_id)?,
        parameters,
        required_files: required_files(measured_repetitions),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: 8192,
        walltime: "02:00:00".to_string(),
        scratch_mb: 8192,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
    };
    Ok(FastqDetectAdaptersMaterial { execution, expectations: expectations.clone() })
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: String::new(),
        negative_exit_nonzero: true,
        metrics: vec![
            metric("sample_count", BenchmarkAcceptanceOperator::Equal, 20, "samples"),
            metric("file_count", BenchmarkAcceptanceOperator::Equal, 30, "files"),
            metric("reads_in", BenchmarkAcceptanceOperator::GreaterThan, 0, "reads"),
            metric("bases_in", BenchmarkAcceptanceOperator::GreaterThan, 0, "bases"),
            metric("mean_q_mean", BenchmarkAcceptanceOperator::GreaterThan, 0, "phred"),
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
