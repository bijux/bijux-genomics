//! Owner: bijux-dna-bench
//! Executable material for the governed FASTQ adapter-inspection campaign.

mod commands;
mod parameters;
mod report;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric,
    BenchmarkAcceptanceOperator, BenchmarkAttemptIdentity, BenchmarkExecutionAcceptance,
    BenchmarkExecutionDeterminism, BenchmarkLifecycleStatus,
};
use bijux_dna_core::contract::ArtifactRole;

use self::commands::{measured_command, negative_command, required_files, warmup_command};
use self::parameters::ensure_parameter_evidence;
use super::{
    fastq_profile_reads::{
        campaign_execution_inputs, canonical_fastq_unit_component, FastqCampaignSample,
    },
    load_attempt_lifecycle, BenchmarkExecutionMaterial, BenchmarkStagePlanInputMaterial,
    BenchmarkStagePlanOutputMaterial, BenchmarkStagePlanUnitMaterial,
    FastqProfileReadsExpectationPublication,
};

pub(crate) use report::publish_fastq_detect_adapters_stage_report_with_presentation_authority;
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
    let stage_plan_units = stage_plan_units(&samples);
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
        stage_plan_units: Some(stage_plan_units),
    };
    Ok(FastqDetectAdaptersMaterial { execution, expectations: expectations.clone() })
}

fn stage_plan_units(samples: &[FastqCampaignSample]) -> Vec<BenchmarkStagePlanUnitMaterial> {
    samples
        .iter()
        .flat_map(|sample| {
            sample.mounted_fastq.iter().enumerate().map(move |(index, mounted_fastq)| {
                BenchmarkStagePlanUnitMaterial {
                    execution_unit_id: format!(
                        "{}-read-{}",
                        canonical_fastq_unit_component(&sample.sample_id),
                        index + 1
                    ),
                    inputs: vec![BenchmarkStagePlanInputMaterial {
                        binding_name: "reads_r1".to_owned(),
                        path: PathBuf::from(mounted_fastq),
                        role: ArtifactRole::Reads,
                    }],
                    outputs: vec![BenchmarkStagePlanOutputMaterial {
                        binding_name: "adapter_evidence_dir".to_owned(),
                        relative_path: "adapter-evidence".into(),
                        role: ArtifactRole::Evidence,
                    }],
                    scientific_defaults: BTreeMap::from([("threads".to_owned(), "4".to_owned())]),
                    input_derived: BTreeMap::new(),
                    operator_overrides: BTreeMap::new(),
                    immutable_parameters: BTreeSet::from(["threads".to_owned()]),
                }
            })
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use bijux_dna_core::ids::{StageId, ToolId};
    use bijux_dna_stage_contract::canonical_tool_bindings;

    use super::{stage_plan_units, FastqCampaignSample, STAGE_ID};

    #[test]
    fn stage_plan_units_preserve_native_fastqc_evidence_per_file() {
        let samples = [FastqCampaignSample {
            sample_id: "ERR-ONE".to_owned(),
            input_layout: "single_end",
            mounted_fastq: vec!["/input/one.fastq.gz".to_owned()],
            expected_read_count: 10,
        }];
        let units = stage_plan_units(&samples);
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].execution_unit_id, "err-one-read-1");
        assert_eq!(units[0].outputs[0].binding_name, "adapter_evidence_dir");
        assert_eq!(units[0].scientific_defaults["threads"], "4");
        assert!(units[0].immutable_parameters.contains("threads"));
    }

    #[test]
    fn stage_plan_units_match_the_canonical_fastqc_binding() -> anyhow::Result<()> {
        let units = stage_plan_units(&[FastqCampaignSample {
            sample_id: "ERR-ONE".to_owned(),
            input_layout: "single_end",
            mounted_fastq: vec!["/input/one.fastq.gz".to_owned()],
            expected_read_count: 10,
        }]);
        let bindings = canonical_tool_bindings()?;
        let binding = bindings
            .binding(&StageId::new(STAGE_ID), &ToolId::new("fastqc"))
            .expect("canonical detect-adapters binding");
        assert_eq!(
            units[0].outputs.iter().map(|output| output.binding_name.as_str()).collect::<Vec<_>>(),
            binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>()
        );
        assert_eq!(
            units[0].scientific_defaults.keys().map(String::as_str).collect::<Vec<_>>(),
            binding.parameters.iter().map(|parameter| parameter.name.as_str()).collect::<Vec<_>>()
        );
        Ok(())
    }
}
