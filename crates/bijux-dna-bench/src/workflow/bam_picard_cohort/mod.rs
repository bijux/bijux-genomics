//! Owner: bijux-dna-bench
//! Executable material and reports for stratified Picard BAM cohort campaigns.

mod commands;
mod report;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, AdmittedBamCohort, AdmittedReferenceBundle,
    BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator,
    BenchmarkAttemptIdentity, BenchmarkEffectiveParameterValue as EffectiveValue,
    BenchmarkExecutionAcceptance, BenchmarkExecutionDeterminism, BenchmarkLifecycleStatus,
};

use self::commands::{
    measured_command, negative_command, required_files, warmup_command, PicardCohortCommandPolicy,
};
use super::bam_observation::{
    cohort_execution_inputs, ensure_cohort_parameter_evidence, BamParameterPolicy,
};
use super::{load_attempt_lifecycle, BenchmarkExecutionMaterial};

pub use report::{publish_bam_gc_bias_stage_report, publish_bam_insert_size_stage_report};
pub(crate) use report::{
    publish_bam_gc_bias_stage_report_with_presentation_authority,
    publish_bam_insert_size_stage_report_with_presentation_authority,
};

const CORPUS_ID: &str = "ancient-dna-bam-diversity";
const REFERENCE_ID: &str = "grch38-reference";
const EXPECTED_SAMPLE_COUNT: usize = 10;

const GC_BIAS_METRICS: &[&str] = &[
    "gc_bias_score",
    "at_dropout",
    "gc_dropout",
    "total_clusters",
    "aligned_reads",
    "windows",
    "read_starts",
];
const INSERT_SIZE_METRICS: &[&str] = &[
    "median_insert_size",
    "mean_insert_size",
    "standard_deviation",
    "median_absolute_deviation",
    "minimum_insert_size",
    "maximum_insert_size",
    "read_pairs",
    "pair_orientation_fr_fraction",
];

#[derive(Debug, Clone, PartialEq)]
pub struct BamPicardCohortMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub admitted_cohort: AdmittedBamCohort,
    pub admitted_reference: Option<AdmittedReferenceBundle>,
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_bam_picard_cohort_material(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    cohort: &AdmittedBamCohort,
    reference: Option<&AdmittedReferenceBundle>,
) -> Result<BamPicardCohortMaterial> {
    validate_attempt_identity(identity).context("validate Picard BAM cohort attempt")?;
    let stage = StagePolicy::for_identity(identity)?;
    if load_attempt_lifecycle(benchmark_root, identity)?.current_status()
        != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("Picard BAM cohort material requires a wholly planned attempt");
    }
    validate_cohort(cohort)?;
    match (stage.requires_reference, reference) {
        (true, Some(reference)) => validate_reference(reference)?,
        (true, None) => bail!("GC-bias campaign requires an admitted reference bundle"),
        (false, Some(_)) => bail!("insert-size campaign does not admit a reference input"),
        (false, None) => {}
    }
    let (inputs, command_samples) = cohort_execution_inputs(cohort, reference)?;
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let parameters = ensure_cohort_parameter_evidence(
        repository_root,
        &attempt_directory,
        cohort,
        BamParameterPolicy { stage_id: stage.stage_id, defaults_provider: stage.defaults_provider },
        BTreeMap::from([
            ("regions".to_string(), EffectiveValue::Null),
            (
                "depth_thresholds".to_string(),
                EffectiveValue::List(vec![EffectiveValue::Integer(1)]),
            ),
            ("regime_mode".to_string(), EffectiveValue::Text("advisory_and_enforced".to_string())),
        ]),
    )?;
    let reference_input_bytes = reference.map_or(0, |value| value.input_bytes);
    let command_policy = PicardCohortCommandPolicy {
        stage_id: stage.stage_id,
        result_stem: stage.result_stem,
        awk_source: stage.awk_source,
        consumer_stage_id: stage.consumer_stage_id,
        input_contract: stage.input_contract,
        scientific_metrics: stage.scientific_metrics,
        reference_input_bytes,
    };
    let execution = BenchmarkExecutionMaterial {
        operator: operator.to_string(),
        prepared_at_utc: prepared_at_utc.to_string(),
        repository_path: PathBuf::from(repository_root),
        benchmark_root: PathBuf::from(benchmark_root),
        source_contract_sha256: source_contract_sha256.clone(),
        inputs,
        warmup_command: warmup_command(&command_policy, &command_samples)?,
        measured_command: measured_command(&command_policy, &command_samples, cohort.input_bytes)?,
        negative_command: negative_command(&command_policy),
        parameters,
        required_files: required_files(&command_policy, &command_samples, 3),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: 16_384,
        walltime: "04:00:00".to_string(),
        scratch_mb: 32_768,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: BenchmarkExecutionAcceptance {
            version_regex: String::new(),
            negative_exit_nonzero: true,
            metrics: vec![BenchmarkAcceptanceMetric {
                id: "sample_count".to_string(),
                path: "repetitions/repetition-1.metrics.json".to_string(),
                json_pointer: "/sample_count".to_string(),
                operator: BenchmarkAcceptanceOperator::Equal,
                expected: BenchmarkAcceptanceExpectedValue::Integer(
                    i64::try_from(EXPECTED_SAMPLE_COUNT).context("sample count")?,
                ),
                unit: "samples".to_string(),
            }],
        },
        stage_plan_units: None,
    };
    Ok(BamPicardCohortMaterial {
        execution,
        admitted_cohort: cohort.clone(),
        admitted_reference: reference.cloned(),
    })
}

fn validate_cohort(cohort: &AdmittedBamCohort) -> Result<()> {
    let input_bytes = cohort.samples.iter().try_fold(0_u64, |total, sample| {
        let expected = sample
            .bam
            .size_bytes
            .checked_add(sample.bam_index.size_bytes)
            .context("Picard BAM cohort sample input overflow")?;
        if sample.input_bytes != expected
            || sample.bam.size_bytes == 0
            || sample.bam_index.size_bytes == 0
        {
            bail!("Picard BAM cohort sample byte identity is invalid");
        }
        total.checked_add(expected).context("Picard BAM cohort input overflow")
    })?;
    if cohort.corpus_id != CORPUS_ID
        || cohort.sample_count != EXPECTED_SAMPLE_COUNT as u64
        || cohort.file_count != (EXPECTED_SAMPLE_COUNT * 2) as u64
        || cohort.samples.len() != EXPECTED_SAMPLE_COUNT
        || cohort.input_bytes != input_bytes
    {
        bail!("admitted BAM cohort differs from the Picard campaign closure");
    }
    Ok(())
}

fn validate_reference(reference: &AdmittedReferenceBundle) -> Result<()> {
    let expected = reference
        .fasta
        .size_bytes
        .checked_add(reference.fasta_index.size_bytes)
        .and_then(|value| value.checked_add(reference.sequence_dictionary.size_bytes))
        .context("Picard reference input overflow")?;
    if reference.reference_id != REFERENCE_ID
        || reference.input_bytes != expected
        || reference.fasta.size_bytes == 0
        || reference.fasta_index.size_bytes == 0
        || reference.sequence_dictionary.size_bytes == 0
    {
        bail!("admitted reference differs from the Picard GC-bias closure");
    }
    Ok(())
}

struct StagePolicy {
    stage_id: &'static str,
    defaults_provider: &'static str,
    result_stem: &'static str,
    awk_source: &'static str,
    consumer_stage_id: &'static str,
    input_contract: &'static str,
    scientific_metrics: &'static [&'static str],
    requires_reference: bool,
}

impl StagePolicy {
    fn for_identity(identity: &BenchmarkAttemptIdentity) -> Result<Self> {
        if identity.run.domain != "bam" || identity.tool_id != "picard" {
            bail!("attempt identity is outside the governed Picard BAM cohort closure");
        }
        match (identity.run.run_id.as_str(), identity.run.stage_id.as_str()) {
            ("run-2013", "bam.gc_bias") => Ok(Self {
                stage_id: "bam.gc_bias",
                defaults_provider:
                    "bijux_dna_domain_bam::stage_specs::stage_spec(BamStage::GcBias)",
                result_stem: "gc-bias-summary",
                awk_source: "bam_gc_bias.awk",
                consumer_stage_id: "benchmark.bam.gc_bias.consume",
                input_contract: "bijux.benchmark.bam.gc_bias.bundle.v1",
                scientific_metrics: GC_BIAS_METRICS,
                requires_reference: true,
            }),
            ("run-2015", "bam.insert_size") => Ok(Self {
                stage_id: "bam.insert_size",
                defaults_provider:
                    "bijux_dna_domain_bam::stage_specs::stage_spec(BamStage::InsertSize)",
                result_stem: "insert-size-summary",
                awk_source: "bam_insert_size.awk",
                consumer_stage_id: "benchmark.bam.insert_size.consume",
                input_contract: "bijux.benchmark.bam.insert_size.bundle.v1",
                scientific_metrics: INSERT_SIZE_METRICS,
                requires_reference: false,
            }),
            _ => bail!("attempt identity is outside run-2013/run-2015"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::{
        AdmittedBamCohort, AdmittedBamCohortSample, AdmittedBamCorpusFile, AdmittedReferenceBundle,
    };

    use super::prepare_bam_picard_cohort_material;

    #[test]
    fn material_closes_both_stratified_picard_campaigns() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let temporary = bijux_dna_infra::temp_dir("picard-cohort-material")?;
        let benchmark_root = temporary.path().join("benchmarks");
        let (registry, _, _) = crate::load_checked_in_benchmark_planning_contracts()?;
        let cohort = admitted_cohort(temporary.path());
        let reference = admitted_reference(temporary.path());
        for run_id in ["run-2013", "run-2015"] {
            let plan = crate::plan_stage_campaign_attempts(
                &registry,
                &"a".repeat(40),
                &"b".repeat(64),
                run_id,
                &format!("{run_id}-cohort"),
                "attempt-001",
                "2026-08-11T00:00:00Z",
            )?;
            crate::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;
            let material = prepare_bam_picard_cohort_material(
                &repository,
                &benchmark_root,
                &plan.cycle.attempts[0].identity,
                &source_contracts(),
                "operator",
                "2026-08-11T00:01:00Z",
                "project",
                "compute",
                &cohort,
                (run_id == "run-2013").then_some(&reference),
            )?;
            assert_eq!(material.execution.inputs.len(), if run_id == "run-2013" { 26 } else { 22 });
            assert!(material.execution.measured_command[2].contains("bam_cohort_summary.awk"));
            assert!(material.execution.measured_command[2].contains("sample-09"));
            assert_eq!(material.execution.acceptance.metrics[0].id, "sample_count");
            assert!(material.execution.required_files.len() > 150);
        }
        Ok(())
    }

    fn source_contracts() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("campaign_registry".to_string(), "b".repeat(64)),
            ("container_catalog".to_string(), "c".repeat(64)),
            ("image_inventory".to_string(), "d".repeat(64)),
            ("stage_interface_registry".to_string(), "e".repeat(64)),
        ])
    }

    fn admitted_cohort(root: &std::path::Path) -> AdmittedBamCohort {
        let samples = (0..10)
            .map(|index| {
                let sample_id = format!("sample-{index:02}");
                AdmittedBamCohortSample {
                    sample_id: sample_id.clone(),
                    bam: file(root, &format!("{sample_id}.bam"), 20),
                    bam_index: file(root, &format!("{sample_id}.bam.bai"), 2),
                    input_bytes: 22,
                }
            })
            .collect();
        AdmittedBamCohort {
            corpus_id: "ancient-dna-bam-diversity".to_string(),
            data_index_path: root.join("data-index.json").display().to_string(),
            data_index_sha256: "a".repeat(64),
            catalog_path: root.join("cohort.json").display().to_string(),
            catalog_sha256: "b".repeat(64),
            sample_count: 10,
            file_count: 20,
            input_bytes: 220,
            samples,
        }
    }

    fn admitted_reference(root: &std::path::Path) -> AdmittedReferenceBundle {
        AdmittedReferenceBundle {
            reference_id: "grch38-reference".to_string(),
            data_index_path: root.join("data-index.json").display().to_string(),
            data_index_sha256: "a".repeat(64),
            catalog_path: root.join("reference.json").display().to_string(),
            catalog_sha256: "c".repeat(64),
            fasta: file(root, "reference.fasta", 100),
            fasta_index: file(root, "reference.fasta.fai", 10),
            sequence_dictionary: file(root, "reference.dict", 5),
            input_bytes: 115,
        }
    }

    fn file(root: &std::path::Path, name: &str, size_bytes: u64) -> AdmittedBamCorpusFile {
        AdmittedBamCorpusFile {
            path: root.join(name).display().to_string(),
            sha256: "f".repeat(64),
            size_bytes,
        }
    }
}
