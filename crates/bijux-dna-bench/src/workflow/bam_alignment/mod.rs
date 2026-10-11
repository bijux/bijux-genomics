//! Owner: bijux-dna-bench
//! Executable material for the controlled run-2005 alignment-truth campaign.

mod commands;
mod inputs;
mod report;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, AdmittedAlignmentTruthSet, AdmittedFastqCorpus,
    AdmittedReferenceBundle, BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric,
    BenchmarkAcceptanceOperator, BenchmarkAttemptIdentity,
    BenchmarkEffectiveParameterValue as EffectiveValue, BenchmarkExecutionAcceptance,
    BenchmarkExecutionDeterminism, BenchmarkLifecycleStatus,
};
use bijux_dna_core::contract::ArtifactRole;

use self::commands::{measured_command, negative_command, required_files, warmup_command};
use self::inputs::{alignment_execution_inputs, AlignmentCommandSample};
use super::bam_observation::ensure_alignment_parameter_evidence;
use super::{
    load_attempt_lifecycle, BenchmarkExecutionMaterial, BenchmarkStagePlanInputMaterial,
    BenchmarkStagePlanOutputMaterial, BenchmarkStagePlanUnitMaterial,
};

pub(crate) use report::publish_bam_alignment_stage_report_with_presentation_authority;
pub use report::{publish_bam_alignment_stage_report, BamAlignmentStageReportPublication};

const RUN_ID: &str = "run-2005";
const STAGE_ID: &str = "bam.align";
const PANEL_ID: &str = "alignment-truth-panel";
const REFERENCE_ID: &str = "grch38-reference";
const EXPECTED_SAMPLE_COUNT: u64 = 10;

#[derive(Debug, Clone, PartialEq)]
pub struct BamAlignmentMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub admitted_corpus: AdmittedFastqCorpus,
    pub admitted_reference: AdmittedReferenceBundle,
    pub admitted_truth: AdmittedAlignmentTruthSet,
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_bam_alignment_material(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    corpus: &AdmittedFastqCorpus,
    reference: &AdmittedReferenceBundle,
    truth: &AdmittedAlignmentTruthSet,
) -> Result<BamAlignmentMaterial> {
    validate_attempt_identity(identity).context("validate BAM alignment attempt")?;
    if identity.run.run_id != RUN_ID
        || identity.run.domain != "bam"
        || identity.run.stage_id != STAGE_ID
        || !matches!(identity.tool_id.as_str(), "bowtie2" | "bwa")
    {
        bail!("attempt identity is outside the governed run-2005 closure");
    }
    if load_attempt_lifecycle(benchmark_root, identity)?.current_status()
        != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("BAM alignment material requires a wholly planned attempt");
    }
    validate_evidence(corpus, reference, truth)?;
    let (inputs, samples) = alignment_execution_inputs(corpus, reference, truth)?;
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let parameters = ensure_alignment_parameter_evidence(
        repository_root,
        &attempt_directory,
        corpus,
        BTreeMap::from([
            (
                "reference".to_string(),
                EffectiveValue::Text("/input/reference/reference.fasta".to_string()),
            ),
            ("reference_digest".to_string(), EffectiveValue::Text(reference.fasta.sha256.clone())),
            ("read_group".to_string(), EffectiveValue::Text("sample_locked".to_string())),
            ("strategy_id".to_string(), EffectiveValue::Text(identity.tool_id.clone())),
            ("preset".to_string(), EffectiveValue::Text("sensitive".to_string())),
            ("mode".to_string(), EffectiveValue::Text("layout_derived".to_string())),
            ("rg_policy".to_string(), EffectiveValue::Text("sample_locked".to_string())),
            (
                "sensitivity_profile".to_string(),
                EffectiveValue::Text("governed_default".to_string()),
            ),
            ("seed_length".to_string(), EffectiveValue::Integer(19)),
            ("build_indices".to_string(), EffectiveValue::Boolean(true)),
            ("emit_stats".to_string(), EffectiveValue::Boolean(true)),
        ]),
    )?;
    let stage_plan_units = stage_plan_units(&identity.tool_id, &samples)?;
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
            reference.input_bytes,
            corpus.input_bytes,
            truth.input_bytes,
        )?,
        negative_command: negative_command(&identity.tool_id),
        parameters,
        required_files: required_files(&samples, 3),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: 32_768,
        walltime: "12:00:00".to_string(),
        scratch_mb: 262_144,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
        stage_plan_units: Some(stage_plan_units),
        downstream_probe_mode: crate::workflow::BenchmarkDownstreamProbeMode::RunnerProduced,
    };
    Ok(BamAlignmentMaterial {
        execution,
        admitted_corpus: corpus.clone(),
        admitted_reference: reference.clone(),
        admitted_truth: truth.clone(),
    })
}

fn stage_plan_units(
    tool_id: &str,
    samples: &[AlignmentCommandSample],
) -> Result<Vec<BenchmarkStagePlanUnitMaterial>> {
    samples
        .iter()
        .map(|sample| {
            let [reads_r1, reads_r2] = sample.fastq_mounts.as_slice() else {
                bail!("BAM alignment stage plans require an admitted paired-end sample");
            };
            let input_derived = match tool_id {
                "bowtie2" => BTreeMap::from([
                    ("read_group_id".to_owned(), sample.sample_id.clone()),
                    ("read_group_sample".to_owned(), sample.sample_id.clone()),
                ]),
                "bwa" => BTreeMap::from([(
                    "read_group".to_owned(),
                    format!(
                        "@RG\\tID:{}\\tSM:{}\\tPL:ILLUMINA",
                        sample.sample_id, sample.sample_id
                    ),
                )]),
                _ => bail!("unsupported BAM alignment stage-plan tool"),
            };
            let immutable_parameters = input_derived
                .keys()
                .cloned()
                .chain(std::iter::once("threads".to_owned()))
                .collect::<BTreeSet<_>>();
            Ok(BenchmarkStagePlanUnitMaterial {
                execution_unit_id: sample
                    .sample_id
                    .to_ascii_lowercase()
                    .chars()
                    .map(
                        |character| {
                            if character.is_ascii_alphanumeric() {
                                character
                            } else {
                                '-'
                            }
                        },
                    )
                    .collect(),
                inputs: vec![
                    stage_input("reads_r1", reads_r1, ArtifactRole::Reads),
                    stage_input("reads_r2", reads_r2, ArtifactRole::Reads),
                    stage_input(
                        "reference",
                        "/input/reference/reference.fasta",
                        ArtifactRole::Reference,
                    ),
                ],
                outputs: vec![BenchmarkStagePlanOutputMaterial {
                    binding_name: "raw_sam".to_owned(),
                    relative_path: "raw.sam".into(),
                    role: ArtifactRole::Alignment,
                }],
                scientific_defaults: BTreeMap::from([("threads".to_owned(), "8".to_owned())]),
                input_derived,
                operator_overrides: BTreeMap::new(),
                immutable_parameters,
            })
        })
        .collect()
}

fn stage_input(
    binding_name: &str,
    path: &str,
    role: ArtifactRole,
) -> BenchmarkStagePlanInputMaterial {
    BenchmarkStagePlanInputMaterial {
        binding_name: binding_name.to_owned(),
        path: PathBuf::from(path),
        role,
    }
}

fn validate_evidence(
    corpus: &AdmittedFastqCorpus,
    reference: &AdmittedReferenceBundle,
    truth: &AdmittedAlignmentTruthSet,
) -> Result<()> {
    let reference_bytes = reference
        .fasta
        .size_bytes
        .checked_add(reference.fasta_index.size_bytes)
        .and_then(|value| value.checked_add(reference.sequence_dictionary.size_bytes))
        .context("alignment reference size overflow")?;
    let truth_bytes = truth.samples.iter().try_fold(0_u64, |total, sample| {
        total.checked_add(sample.truth.size_bytes).context("alignment truth size overflow")
    })?;
    if corpus.corpus_id != PANEL_ID
        || truth.truth_set_id != PANEL_ID
        || reference.reference_id != REFERENCE_ID
        || corpus.sample_count != EXPECTED_SAMPLE_COUNT
        || truth.sample_count != EXPECTED_SAMPLE_COUNT
        || corpus.samples.len() != EXPECTED_SAMPLE_COUNT as usize
        || truth.samples.len() != EXPECTED_SAMPLE_COUNT as usize
        || corpus.input_bytes == 0
        || truth.input_bytes != truth_bytes
        || truth.file_count != EXPECTED_SAMPLE_COUNT
        || reference.input_bytes != reference_bytes
    {
        bail!("admitted evidence differs from the run-2005 alignment closure");
    }
    Ok(())
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: String::new(),
        negative_exit_nonzero: true,
        metrics: vec![
            metric("sample_count", BenchmarkAcceptanceOperator::Equal, 10, "samples"),
            metric("total_reads", BenchmarkAcceptanceOperator::GreaterThan, 0, "reads"),
            metric("expected_mapped_reads", BenchmarkAcceptanceOperator::GreaterThan, 0, "reads"),
            metric("exact_placement_reads", BenchmarkAcceptanceOperator::GreaterThan, 0, "reads"),
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
    use std::collections::{BTreeMap, BTreeSet};

    use bijux_dna_bench_model::{
        AdmittedAlignmentTruthSample, AdmittedAlignmentTruthSet, AdmittedBamCorpusFile,
        AdmittedFastqCorpus, AdmittedFastqCorpusFile, AdmittedFastqCorpusSample,
        AdmittedReferenceBundle, BenchmarkFastqLayout,
    };

    use super::{prepare_bam_alignment_material, stage_plan_units, AlignmentCommandSample};

    #[test]
    fn material_closes_both_run_2005_aligners() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let temporary = bijux_dna_infra::temp_dir("bam-alignment-material")?;
        let benchmark_root = temporary.path().join("benchmarks");
        let (registry, _, _) = crate::load_checked_in_benchmark_planning_contracts()?;
        let plan = crate::plan_stage_campaign_attempts(
            &registry,
            &"a".repeat(40),
            &"b".repeat(64),
            "run-2005",
            "alignment-truth-comparison",
            "attempt-001",
            "2026-08-11T00:00:00Z",
        )?;
        crate::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;
        let (corpus, reference, truth) = evidence(temporary.path());
        for attempt in &plan.cycle.attempts {
            let material = prepare_bam_alignment_material(
                &repository,
                &benchmark_root,
                &attempt.identity,
                &source_contracts(),
                "operator",
                "2026-08-11T00:01:00Z",
                "project",
                "compute",
                &corpus,
                &reference,
                &truth,
            )?;
            assert_eq!(material.execution.acceptance.metrics.len(), 4);
            assert!(material.execution.measured_command[2].contains("alignment-truth-summary.tsv"));
            assert!(material.execution.required_files.len() > 150);
            assert_eq!(material.execution.stage_plan_units.as_ref().map(Vec::len), Some(10));
            let unit = &material.execution.stage_plan_units.as_ref().expect("stage-plan units")[0];
            let bindings = bijux_dna_stage_contract::canonical_tool_bindings()?;
            let binding = bindings
                .binding(
                    &bijux_dna_core::ids::StageId::new(super::STAGE_ID),
                    &bijux_dna_core::ids::ToolId::new(attempt.identity.tool_id.as_str()),
                )
                .expect("canonical BAM alignment binding");
            assert_eq!(
                unit.inputs.iter().map(|input| input.binding_name.as_str()).collect::<Vec<_>>(),
                binding.inputs.iter().map(|input| input.name.as_str()).collect::<Vec<_>>()
            );
            assert_eq!(
                unit.outputs.iter().map(|output| output.binding_name.as_str()).collect::<Vec<_>>(),
                binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>()
            );
            assert_eq!(
                unit.scientific_defaults
                    .keys()
                    .chain(unit.input_derived.keys())
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>(),
                binding.parameters.iter().map(|parameter| parameter.name.as_str()).collect()
            );
        }
        Ok(())
    }

    #[test]
    fn stage_plan_units_reject_single_end_material_before_execution() {
        let samples = [AlignmentCommandSample {
            sample_id: "sample-single".to_owned(),
            layout: BenchmarkFastqLayout::SingleEnd,
            fastq_mounts: vec!["/input/panel/sample-single.reads.fastq.gz".to_owned()],
            fastq_input_bytes: 10,
            truth_mount: "/input/truth/sample-single.tsv".to_owned(),
            truth_input_bytes: 3,
        }];
        assert!(stage_plan_units("bowtie2", &samples).is_err());
        assert!(stage_plan_units("bwa", &samples).is_err());
    }

    fn source_contracts() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("campaign_registry".to_string(), "b".repeat(64)),
            ("container_catalog".to_string(), "c".repeat(64)),
            ("image_inventory".to_string(), "d".repeat(64)),
            ("stage_interface_registry".to_string(), "e".repeat(64)),
        ])
    }

    fn evidence(
        root: &std::path::Path,
    ) -> (AdmittedFastqCorpus, AdmittedReferenceBundle, AdmittedAlignmentTruthSet) {
        let data_index_path = root.join("data-index.json").display().to_string();
        let samples = (0..10)
            .map(|index| {
                let sample_id = format!("sample-{index:02}");
                AdmittedFastqCorpusSample {
                    sample_id: sample_id.clone(),
                    input_layout: BenchmarkFastqLayout::PairedEnd,
                    files: vec![
                        fastq(root, &format!("{sample_id}/R1.fastq.gz"), 10, 1),
                        fastq(root, &format!("{sample_id}/R2.fastq.gz"), 10, 2),
                    ],
                }
            })
            .collect::<Vec<_>>();
        let truth_samples = (0..10)
            .map(|index| {
                let sample_id = format!("sample-{index:02}");
                AdmittedAlignmentTruthSample {
                    sample_id: sample_id.clone(),
                    truth: file(root, &format!("{sample_id}/truth.tsv"), 3),
                }
            })
            .collect();
        (
            AdmittedFastqCorpus {
                corpus_id: "alignment-truth-panel".to_string(),
                data_index_path: data_index_path.clone(),
                data_index_sha256: "a".repeat(64),
                catalog_path: root.join("panel.json").display().to_string(),
                catalog_sha256: "b".repeat(64),
                sample_count: 10,
                file_count: 20,
                input_bytes: 200,
                samples,
            },
            AdmittedReferenceBundle {
                reference_id: "grch38-reference".to_string(),
                data_index_path: data_index_path.clone(),
                data_index_sha256: "a".repeat(64),
                catalog_path: root.join("reference.json").display().to_string(),
                catalog_sha256: "c".repeat(64),
                fasta: file(root, "reference.fasta", 100),
                fasta_index: file(root, "reference.fasta.fai", 10),
                sequence_dictionary: file(root, "reference.dict", 5),
                input_bytes: 115,
            },
            AdmittedAlignmentTruthSet {
                truth_set_id: "alignment-truth-panel".to_string(),
                data_index_path,
                data_index_sha256: "a".repeat(64),
                catalog_path: root.join("panel.json").display().to_string(),
                catalog_sha256: "b".repeat(64),
                sample_count: 10,
                file_count: 10,
                input_bytes: 30,
                samples: truth_samples,
            },
        )
    }

    fn fastq(
        root: &std::path::Path,
        name: &str,
        size_bytes: u64,
        mate: u8,
    ) -> AdmittedFastqCorpusFile {
        AdmittedFastqCorpusFile {
            path: root.join(name).display().to_string(),
            sha256: "d".repeat(64),
            size_bytes,
            mate: Some(mate),
        }
    }

    fn file(root: &std::path::Path, name: &str, size_bytes: u64) -> AdmittedBamCorpusFile {
        AdmittedBamCorpusFile {
            path: root.join(name).display().to_string(),
            sha256: "e".repeat(64),
            size_bytes,
        }
    }
}
