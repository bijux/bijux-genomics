//! Owner: bijux-dna-bench
//! Executable material for the governed FASTQ read-profile campaign.

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
    ensure_fastq_profile_reads_expectations, ensure_fastq_read_length_expectations,
    load_attempt_lifecycle, BenchmarkExecutionMaterial, FastqProfileReadsExpectationPublication,
};

const RUN_ID: &str = "run-2043";
const STAGE_ID: &str = "fastq.profile_reads";
const CORPUS_ID: &str = "human-short-read-diversity";

#[derive(Debug, Clone, PartialEq)]
pub struct FastqProfileReadsMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub expectations: FastqProfileReadsExpectationPublication,
}

#[allow(clippy::too_many_arguments)]
/// Prepare complete executable material for one planned run-2043 tool attempt.
///
/// # Errors
/// Returns an error when attempt state, source locks, corpus truth, parameters, commands, or
/// acceptance closure differs from the governed campaign.
pub fn prepare_fastq_profile_reads_material(
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
) -> Result<FastqProfileReadsMaterial> {
    let read_lengths = ensure_fastq_read_length_expectations(
        repository_root,
        study_root,
        data_index_path,
        CORPUS_ID,
        20,
        30,
    )?;
    let expectations = ensure_fastq_profile_reads_expectations(study_root, &read_lengths)?;
    prepare_fastq_profile_reads_material_from_expectations(
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
/// Prepare one run-2043 tool attempt from campaign-wide immutable expectations.
///
/// # Errors
/// Returns an error when the shared expectation publication or attempt authorities disagree.
pub fn prepare_fastq_profile_reads_material_from_expectations(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    expectations: &FastqProfileReadsExpectationPublication,
) -> Result<FastqProfileReadsMaterial> {
    validate_attempt_identity(identity).context("validate FASTQ profile attempt identity")?;
    if identity.run.run_id != RUN_ID
        || identity.run.domain != "fastq"
        || identity.run.stage_id != STAGE_ID
        || !matches!(identity.tool_id.as_str(), "seqfu" | "seqkit" | "seqkit_stats")
    {
        bail!("attempt identity is outside the governed run-2043 closure");
    }
    let lifecycle = load_attempt_lifecycle(benchmark_root, identity)?;
    if lifecycle.current_status() != Some(BenchmarkLifecycleStatus::Planned) {
        bail!("FASTQ profile execution material requires a wholly planned attempt");
    }
    if expectations.admitted_corpus.corpus_id != CORPUS_ID
        || expectations.admitted_corpus.sample_count != 20
        || expectations.admitted_corpus.file_count != 30
        || expectations.samples.len() != 20
    {
        bail!("FASTQ profile expectation closure differs from the campaign");
    }
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let parameters = ensure_parameter_evidence(
        repository_root,
        &attempt_directory,
        &expectations.admitted_corpus,
    )?;
    let (inputs, mounted_fastq) = execution_inputs(expectations)?;
    let measured_repetitions = 3;
    let execution = BenchmarkExecutionMaterial {
        operator: operator.to_string(),
        prepared_at_utc: prepared_at_utc.to_string(),
        repository_path: PathBuf::from(repository_root),
        benchmark_root: PathBuf::from(benchmark_root),
        source_contract_sha256: source_contract_sha256.clone(),
        inputs,
        warmup_command: warmup_command(&identity.tool_id, &mounted_fastq),
        measured_command: measured_command(
            &identity.tool_id,
            &mounted_fastq,
            expectations.admitted_corpus.input_bytes,
        ),
        negative_command: negative_command(&identity.tool_id),
        parameters,
        required_files: required_files(measured_repetitions),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: 8192,
        walltime: "01:00:00".to_string(),
        scratch_mb: 4096,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
    };
    Ok(FastqProfileReadsMaterial { execution, expectations: expectations.clone() })
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: String::new(),
        negative_exit_nonzero: true,
        metrics: vec![
            metric("sample_count", BenchmarkAcceptanceOperator::Equal, 20, "samples"),
            metric("file_count", BenchmarkAcceptanceOperator::Equal, 30, "files"),
            metric("read_count", BenchmarkAcceptanceOperator::GreaterThan, 0, "reads"),
            metric("base_count", BenchmarkAcceptanceOperator::GreaterThan, 0, "bases"),
            metric("admission_mismatch_count", BenchmarkAcceptanceOperator::Equal, 0, "samples"),
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
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::{
        AdmittedFastqCorpus, AdmittedFastqCorpusFile, AdmittedFastqCorpusSample,
        BenchmarkFastqLayout,
    };

    use super::{
        prepare_fastq_profile_reads_material_from_expectations,
        FastqProfileReadsExpectationPublication,
    };
    use crate::FastqProfileReadsExpectationSample;

    #[test]
    fn material_closes_all_three_run_2043_tools() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let temporary = bijux_dna_infra::temp_dir("fastq-profile-material")?;
        let benchmark_root = temporary.path().join("benchmarks");
        let (registry, _, _) = crate::load_checked_in_benchmark_planning_contracts()?;
        let plan = crate::plan_stage_campaign_attempts(
            &registry,
            &"a".repeat(40),
            &"b".repeat(64),
            "run-2043",
            "human-short-read-profile-comparison",
            "attempt-001",
            "2026-08-05T00:00:00Z",
        )?;
        crate::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;
        let expectations = publication(temporary.path())?;
        let source = BTreeMap::from([
            ("campaign_registry".to_string(), "b".repeat(64)),
            ("container_catalog".to_string(), "c".repeat(64)),
            ("image_inventory".to_string(), "d".repeat(64)),
            ("stage_interface_registry".to_string(), "e".repeat(64)),
        ]);
        for attempt in &plan.cycle.attempts {
            let material = prepare_fastq_profile_reads_material_from_expectations(
                &repository,
                &benchmark_root,
                &attempt.identity,
                &source,
                "operator",
                "2026-08-05T00:01:00Z",
                "project",
                "compute",
                &expectations,
            )?;
            assert!(material.execution.measured_command[2].contains("fastq_profile_reads.awk"));
            assert_eq!(material.execution.required_files.len(), 21);
            assert_eq!(material.execution.acceptance.metrics.len(), 5);
        }
        Ok(())
    }

    fn publication(
        root: &std::path::Path,
    ) -> anyhow::Result<FastqProfileReadsExpectationPublication> {
        let evidence_root = root.join("expectations");
        std::fs::create_dir_all(&evidence_root)?;
        let profile_path = evidence_root.join("expected-profile.tsv");
        let provenance_path = evidence_root.join("provenance.json");
        std::fs::write(&profile_path, b"profile\n")?;
        std::fs::write(&provenance_path, b"{}\n")?;
        let mut samples = Vec::new();
        let mut admitted_samples = Vec::new();
        for index in 0..20 {
            let paired = index < 10;
            let sample_id = format!("ERR{index:06}");
            let input_layout = if paired {
                BenchmarkFastqLayout::PairedEnd
            } else {
                BenchmarkFastqLayout::SingleEnd
            };
            let file_count = if paired { 2 } else { 1 };
            let files = (1..=file_count)
                .map(|mate| AdmittedFastqCorpusFile {
                    path: root.join(format!("{sample_id}_{mate}.fastq.gz")).display().to_string(),
                    sha256: format!("{index:064x}"),
                    size_bytes: 1,
                    mate: paired.then_some(mate as u8),
                })
                .collect();
            samples.push(FastqProfileReadsExpectationSample {
                sample_id: sample_id.clone(),
                input_layout,
                file_count,
                read_count: 1,
                base_count: 1,
            });
            admitted_samples.push(AdmittedFastqCorpusSample { sample_id, input_layout, files });
        }
        Ok(FastqProfileReadsExpectationPublication {
            root: evidence_root,
            profile_path,
            provenance_path,
            admitted_corpus: AdmittedFastqCorpus {
                corpus_id: "human-short-read-diversity".to_string(),
                data_index_path: root.join("data-index.json").display().to_string(),
                data_index_sha256: "1".repeat(64),
                catalog_path: root.join("catalog.json").display().to_string(),
                catalog_sha256: "2".repeat(64),
                sample_count: 20,
                file_count: 30,
                input_bytes: 30,
                samples: admitted_samples,
            },
            samples,
        })
    }
}
