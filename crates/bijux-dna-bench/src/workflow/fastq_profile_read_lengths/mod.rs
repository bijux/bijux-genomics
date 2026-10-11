//! Owner: bijux-dna-bench
//! Executable material for the governed FASTQ read-length campaign.

mod commands;
mod inputs;
mod parameters;
mod report;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, validate_fastq_read_length_expectations,
    BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator,
    BenchmarkAttemptIdentity, BenchmarkExecutionAcceptance, BenchmarkExecutionDeterminism,
    BenchmarkLifecycleStatus,
};
use bijux_dna_core::contract::ArtifactRole;

use self::commands::{measured_command, negative_command, required_files, warmup_command};
use self::inputs::{execution_inputs, CommandSample};
use self::parameters::ensure_parameter_evidence;
use super::{
    ensure_fastq_read_length_expectations, load_attempt_lifecycle, BenchmarkExecutionMaterial,
    BenchmarkStagePlanInputMaterial, BenchmarkStagePlanOutputMaterial,
    BenchmarkStagePlanUnitMaterial, FastqReadLengthExpectationPublication,
};

pub(crate) use report::publish_fastq_profile_read_lengths_stage_report_with_presentation_authority;
pub use report::{
    publish_fastq_profile_read_lengths_stage_report, render_fastq_profile_read_lengths_markdown,
    FastqProfileReadLengthsStageReportPublication,
};

const RUN_ID: &str = "run-2042";
const STAGE_ID: &str = "fastq.profile_read_lengths";
const CORPUS_ID: &str = "human-short-read-diversity";
const SEQFU_WALLTIME: &str = "01:30:00";
const SEQKIT_WALLTIME: &str = "01:00:00";

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
    let expectations = ensure_fastq_read_length_expectations(
        repository_root,
        study_root,
        data_index_path,
        CORPUS_ID,
        20,
        30,
    )?;
    prepare_fastq_profile_read_lengths_material_from_expectations(
        repository_root,
        benchmark_root,
        identity,
        source_contract_sha256,
        operator,
        prepared_at_utc,
        account,
        partition,
        &expectations,
    )
}

#[allow(clippy::too_many_arguments)]
/// Prepare one tool attempt from campaign-wide expectations admitted by the same invocation.
///
/// This keeps expensive corpus hashing and independent truth derivation at campaign ownership
/// rather than repeating it for every tool while preserving the same immutable input closure.
///
/// # Errors
/// Returns an error when the expectation publication or attempt authorities are inconsistent.
pub fn prepare_fastq_profile_read_lengths_material_from_expectations(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    expectations: &FastqReadLengthExpectationPublication,
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
    validate_fastq_read_length_expectations(&expectations.expectations)
        .context("validate shared FASTQ read-length expectations")?;
    if expectations.expectations.corpus_id != CORPUS_ID
        || expectations.admitted_corpus.corpus_id != CORPUS_ID
        || expectations.expectations.data_index_sha256
            != expectations.admitted_corpus.data_index_sha256
        || expectations.expectations.catalog_sha256 != expectations.admitted_corpus.catalog_sha256
        || expectations.expectations.sample_count != expectations.admitted_corpus.sample_count
        || expectations.expectations.file_count != expectations.admitted_corpus.file_count
    {
        bail!("shared FASTQ expectation and admitted corpus authorities differ");
    }
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let parameters = ensure_parameter_evidence(
        repository_root,
        &attempt_directory,
        &expectations.admitted_corpus,
    )?;
    let (inputs, samples) = execution_inputs(expectations)?;
    let stage_plan_units = stage_plan_units(&identity.tool_id, &samples);
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
        walltime: execution_walltime(&identity.tool_id).to_string(),
        scratch_mb: 4096,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
        stage_plan_units: Some(stage_plan_units),
        downstream_probe_mode: crate::workflow::BenchmarkDownstreamProbeMode::RunnerProduced,
    };
    Ok(FastqProfileReadLengthsMaterial { execution, expectations: expectations.clone() })
}

fn execution_walltime(tool_id: &str) -> &'static str {
    if tool_id == "seqfu" {
        SEQFU_WALLTIME
    } else {
        SEQKIT_WALLTIME
    }
}

fn stage_plan_units(
    tool_id: &str,
    samples: &[CommandSample],
) -> Vec<BenchmarkStagePlanUnitMaterial> {
    let (scientific_defaults, immutable_parameters) = if tool_id == "seqkit" {
        (
            BTreeMap::from([("threads".to_owned(), "4".to_owned())]),
            BTreeSet::from(["threads".to_owned()]),
        )
    } else {
        (BTreeMap::new(), BTreeSet::new())
    };
    samples
        .iter()
        .flat_map(|sample| {
            let scientific_defaults = scientific_defaults.clone();
            let immutable_parameters = immutable_parameters.clone();
            sample.mounted_fastq.iter().enumerate().map(move |(index, mounted_fastq)| {
                BenchmarkStagePlanUnitMaterial {
                    execution_unit_id: format!(
                        "{}-read-{}",
                        canonical_unit_component(&sample.sample_id),
                        index + 1
                    ),
                    inputs: vec![BenchmarkStagePlanInputMaterial {
                        binding_name: "reads_r1".to_owned(),
                        path: PathBuf::from(mounted_fastq),
                        role: ArtifactRole::Reads,
                    }],
                    outputs: vec![BenchmarkStagePlanOutputMaterial {
                        binding_name: "length_distribution_tsv".to_owned(),
                        relative_path: "length-distribution.tsv".into(),
                        role: ArtifactRole::SummaryTsv,
                    }],
                    scientific_defaults: scientific_defaults.clone(),
                    input_derived: BTreeMap::new(),
                    operator_overrides: BTreeMap::new(),
                    immutable_parameters: immutable_parameters.clone(),
                }
            })
        })
        .collect()
}

fn canonical_unit_component(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '-' })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::{execution_walltime, stage_plan_units, CommandSample};

    #[test]
    fn production_walltime_is_bounded_per_tool() {
        assert_eq!(execution_walltime("seqfu"), "01:30:00");
        assert_eq!(execution_walltime("seqkit"), "01:00:00");
    }

    #[test]
    fn stage_plan_units_close_each_admitted_fastq_file() {
        let samples = [CommandSample {
            sample_id: "ERR-ONE".to_owned(),
            input_layout: "paired_end",
            mounted_fastq: vec![
                "/input/one-r1.fastq.gz".to_owned(),
                "/input/one-r2.fastq.gz".to_owned(),
            ],
            expected_summary_mount: "/input/one.summary.tsv".to_owned(),
            expected_histogram_mount: "/input/one.histogram.tsv".to_owned(),
            expected_exact_sha256: "a".repeat(64),
        }];
        let units = stage_plan_units("seqkit", &samples);
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].execution_unit_id, "err-one-read-1");
        assert_eq!(units[1].execution_unit_id, "err-one-read-2");
        assert_eq!(units[0].scientific_defaults["threads"], "4");
        assert!(units[0].immutable_parameters.contains("threads"));
        assert_eq!(units[0].inputs[0].path.to_string_lossy(), "/input/one-r1.fastq.gz");
        assert_eq!(units[0].outputs[0].relative_path.to_string_lossy(), "length-distribution.tsv");

        let seqfu = stage_plan_units("seqfu", &samples);
        assert!(seqfu.iter().all(|unit| unit.scientific_defaults.is_empty()));
    }
}
