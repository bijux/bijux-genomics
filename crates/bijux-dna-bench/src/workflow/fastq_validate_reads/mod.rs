//! Owner: bijux-dna-bench
//! Executable material for the governed FASTQ validation campaign.

mod commands;
mod parameters;
mod planner_contract;
mod report;

use std::collections::BTreeMap;
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

pub(crate) use report::publish_fastq_validate_reads_stage_report_with_presentation_authority;
pub use report::{
    publish_fastq_validate_reads_stage_report, render_fastq_validate_reads_markdown,
    FastqValidateReadsStageReportPublication,
};

const RUN_ID: &str = "run-2051";
const STAGE_ID: &str = "fastq.validate_reads";
const CORPUS_ID: &str = "human-short-read-diversity";

#[derive(Debug, Clone, PartialEq)]
pub struct FastqValidateReadsMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub expectations: FastqProfileReadsExpectationPublication,
}

#[allow(clippy::too_many_arguments)]
/// Prepare one planned run-2051 tool attempt from campaign-wide immutable expectations.
///
/// # Errors
/// Returns an error when attempt state, corpus truth, planner commands, parameters, or acceptance
/// closure differs from the governed validation campaign.
pub fn prepare_fastq_validate_reads_material_from_expectations(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    expectations: &FastqProfileReadsExpectationPublication,
) -> Result<FastqValidateReadsMaterial> {
    validate_attempt_identity(identity).context("validate FASTQ validation attempt identity")?;
    if identity.run.run_id != RUN_ID
        || identity.run.domain != "fastq"
        || identity.run.stage_id != STAGE_ID
        || !matches!(
            identity.tool_id.as_str(),
            "fastq_scan" | "fastqc" | "fastqvalidator" | "fqtools" | "seqtk"
        )
    {
        bail!("attempt identity is outside the governed run-2051 closure");
    }
    let lifecycle = load_attempt_lifecycle(benchmark_root, identity)?;
    if lifecycle.current_status() != Some(BenchmarkLifecycleStatus::Planned) {
        bail!("FASTQ validation material requires a wholly planned attempt");
    }
    if expectations.admitted_corpus.corpus_id != CORPUS_ID
        || expectations.admitted_corpus.sample_count != 20
        || expectations.admitted_corpus.file_count != 30
        || expectations.samples.len() != 20
    {
        bail!("FASTQ validation expectation closure differs from the campaign");
    }
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let (inputs, samples) = campaign_execution_inputs(expectations)?;
    let parameters = ensure_parameter_evidence(repository_root, &attempt_directory, &samples)?;
    let stage_plan_units = stage_plan_units(&identity.tool_id, &samples)?;
    let measured_repetitions = 3;
    let execution = BenchmarkExecutionMaterial {
        operator: operator.to_string(),
        prepared_at_utc: prepared_at_utc.to_string(),
        repository_path: PathBuf::from(repository_root),
        benchmark_root: PathBuf::from(benchmark_root),
        source_contract_sha256: source_contract_sha256.clone(),
        inputs,
        warmup_command: warmup_command(repository_root, &identity.tool_id, &samples)?,
        measured_command: measured_command(
            repository_root,
            &identity.tool_id,
            &samples,
            expectations.admitted_corpus.input_bytes,
        )?,
        negative_command: negative_command(repository_root, &identity.tool_id)?,
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
        downstream_probe_mode: crate::workflow::BenchmarkDownstreamProbeMode::RunnerProduced,
    };
    Ok(FastqValidateReadsMaterial { execution, expectations: expectations.clone() })
}

fn stage_plan_units(
    tool_id: &str,
    samples: &[FastqCampaignSample],
) -> Result<Vec<BenchmarkStagePlanUnitMaterial>> {
    let outputs = match tool_id {
        "fastqc" => vec![BenchmarkStagePlanOutputMaterial {
            binding_name: "validation_evidence".to_owned(),
            relative_path: "validation-evidence".into(),
            role: ArtifactRole::Evidence,
        }],
        "fastq_scan" => vec![BenchmarkStagePlanOutputMaterial {
            binding_name: "validation_evidence".to_owned(),
            relative_path: "validation-evidence.tsv".into(),
            role: ArtifactRole::SummaryTsv,
        }],
        "fastqvalidator" | "fqtools" | "seqtk" => vec![
            BenchmarkStagePlanOutputMaterial {
                binding_name: "validation_report".to_owned(),
                relative_path: "validation-report.json".into(),
                role: ArtifactRole::ReportJson,
            },
            BenchmarkStagePlanOutputMaterial {
                binding_name: "validated_reads_manifest".to_owned(),
                relative_path: "validated-reads-manifest.json".into(),
                role: ArtifactRole::Provenance,
            },
        ],
        _ => bail!("unsupported FASTQ validation stage-plan tool"),
    };
    Ok(samples
        .iter()
        .flat_map(|sample| {
            let outputs = outputs.clone();
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
                    outputs: outputs.clone(),
                    scientific_defaults: BTreeMap::new(),
                    input_derived: BTreeMap::new(),
                    operator_overrides: BTreeMap::new(),
                    immutable_parameters: std::collections::BTreeSet::new(),
                }
            })
        })
        .collect())
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: String::new(),
        negative_exit_nonzero: true,
        metrics: vec![
            metric("sample_count", BenchmarkAcceptanceOperator::Equal, 20, "samples"),
            metric("file_count", BenchmarkAcceptanceOperator::Equal, 30, "files"),
            metric("validated_read_count", BenchmarkAcceptanceOperator::GreaterThan, 0, "reads"),
            metric("strict_pass_count", BenchmarkAcceptanceOperator::Equal, 20, "samples"),
            metric("pair_sync_failure_count", BenchmarkAcceptanceOperator::Equal, 0, "samples"),
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

    fn sample() -> FastqCampaignSample {
        FastqCampaignSample {
            sample_id: "ERR-ONE".to_owned(),
            input_layout: "single_end",
            mounted_fastq: vec!["/input/one.fastq.gz".to_owned()],
            expected_read_count: 10,
        }
    }

    #[test]
    fn stage_plan_units_preserve_each_canonical_validation_output_contract() -> anyhow::Result<()> {
        let bindings = canonical_tool_bindings()?;
        for tool in ["fastqc", "fastq_scan"] {
            let units = stage_plan_units(tool, &[sample()])?;
            assert_eq!(units[0].outputs[0].binding_name, "validation_evidence");
            assert_binding_outputs(&bindings, tool, &units[0])?;
        }
        for tool in ["fastqvalidator", "fqtools", "seqtk"] {
            let units = stage_plan_units(tool, &[sample()])?;
            assert_eq!(units[0].outputs.len(), 2);
            assert_eq!(units[0].outputs[0].binding_name, "validation_report");
            assert_eq!(units[0].outputs[1].binding_name, "validated_reads_manifest");
            assert_binding_outputs(&bindings, tool, &units[0])?;
        }
        assert!(stage_plan_units("unknown", &[sample()]).is_err());
        Ok(())
    }

    fn assert_binding_outputs(
        bindings: &bijux_dna_stage_contract::ToolBindingRegistry,
        tool_id: &str,
        unit: &crate::BenchmarkStagePlanUnitMaterial,
    ) -> anyhow::Result<()> {
        let binding = bindings
            .binding(&StageId::new(STAGE_ID), &ToolId::new(tool_id))
            .expect("canonical validation binding");
        assert_eq!(
            unit.outputs.iter().map(|output| output.binding_name.as_str()).collect::<Vec<_>>(),
            binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>()
        );
        Ok(())
    }
}
