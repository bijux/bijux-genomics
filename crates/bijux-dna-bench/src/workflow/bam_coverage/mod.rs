//! Owner: bijux-dna-bench
//! Executable material for the governed BAM coverage campaign.

mod commands;
mod reference_span;
mod report;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, AdmittedBamCorpus, BenchmarkAcceptanceExpectedValue,
    BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator, BenchmarkAttemptIdentity,
    BenchmarkEffectiveParameterValue as EffectiveValue, BenchmarkExecutionAcceptance,
    BenchmarkExecutionDeterminism, BenchmarkLifecycleStatus,
};

use self::commands::{measured_command, negative_command, required_files, warmup_command};
use self::reference_span::ensure_reference_span_input;
use super::bam_observation::{ensure_parameter_evidence, execution_inputs, BamParameterPolicy};
use super::{load_attempt_lifecycle, BenchmarkExecutionMaterial};

pub(crate) use report::publish_bam_coverage_stage_report_with_presentation_authority;
pub use report::{
    publish_bam_coverage_stage_report, render_bam_coverage_markdown,
    BamCoverageStageReportPublication,
};

const RUN_ID: &str = "run-2002";
const STAGE_ID: &str = "bam.coverage";
const CORPUS_ID: &str = "ena-prjeb22390-err2112767-bam";
const SAMPLE_ID: &str = "ERR2112767";

#[derive(Debug, Clone, PartialEq)]
pub struct BamCoverageMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub admitted_corpus: AdmittedBamCorpus,
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_bam_coverage_material(
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
) -> Result<BamCoverageMaterial> {
    let corpus = crate::admit_bam_benchmark_corpus(study_root, data_index_path, CORPUS_ID)?;
    prepare_bam_coverage_material_from_corpus(
        repository_root,
        benchmark_root,
        identity,
        source_contract_sha256,
        operator,
        prepared_at_utc,
        account,
        partition,
        &corpus,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_bam_coverage_material_from_corpus(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    corpus: &AdmittedBamCorpus,
) -> Result<BamCoverageMaterial> {
    validate_attempt_identity(identity).context("validate BAM coverage attempt identity")?;
    if identity.run.run_id != RUN_ID
        || identity.run.domain != "bam"
        || identity.run.stage_id != STAGE_ID
        || !matches!(identity.tool_id.as_str(), "bedtools" | "mosdepth" | "samtools")
    {
        bail!("attempt identity is outside the governed run-2002 closure");
    }
    require_planned_attempt(benchmark_root, identity)?;
    validate_corpus(corpus)?;
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let parameters = ensure_parameter_evidence(
        repository_root,
        &attempt_directory,
        &corpus.sample_id,
        BamParameterPolicy {
            stage_id: STAGE_ID,
            defaults_provider: "bijux_dna_domain_bam::stage_specs::stage_spec(BamStage::Coverage)",
        },
        BTreeMap::from([
            ("regions".to_string(), EffectiveValue::Null),
            (
                "depth_thresholds".to_string(),
                EffectiveValue::List(vec![
                    EffectiveValue::Integer(1),
                    EffectiveValue::Integer(3),
                    EffectiveValue::Integer(5),
                ]),
            ),
            ("regime_mode".to_string(), EffectiveValue::Text("advisory_and_enforced".to_string())),
        ]),
    )?;
    let mut inputs = execution_inputs(corpus);
    inputs.push(ensure_reference_span_input(&attempt_directory, corpus)?);
    let execution = BenchmarkExecutionMaterial {
        operator: operator.to_string(),
        prepared_at_utc: prepared_at_utc.to_string(),
        repository_path: PathBuf::from(repository_root),
        benchmark_root: PathBuf::from(benchmark_root),
        source_contract_sha256: source_contract_sha256.clone(),
        inputs,
        warmup_command: warmup_command(&identity.tool_id),
        measured_command: measured_command(
            &identity.tool_id,
            &corpus.sample_id,
            corpus.bam.size_bytes,
            corpus.bam_index.size_bytes,
            corpus.input_bytes,
        ),
        negative_command: negative_command(&identity.tool_id),
        parameters,
        required_files: required_files(&identity.tool_id, 3),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: 8192,
        walltime: "00:45:00".to_string(),
        scratch_mb: 8192,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
        stage_plan_units: None,
    };
    Ok(BamCoverageMaterial { execution, admitted_corpus: corpus.clone() })
}

fn require_planned_attempt(root: &Path, identity: &BenchmarkAttemptIdentity) -> Result<()> {
    if load_attempt_lifecycle(root, identity)?.current_status()
        != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("BAM coverage material requires a wholly planned attempt");
    }
    Ok(())
}

fn validate_corpus(corpus: &AdmittedBamCorpus) -> Result<()> {
    let expected = corpus
        .bam
        .size_bytes
        .checked_add(corpus.bam_index.size_bytes)
        .context("BAM corpus size overflow")?;
    if corpus.corpus_id != CORPUS_ID
        || corpus.sample_id != SAMPLE_ID
        || corpus.bam.size_bytes == 0
        || corpus.bam_index.size_bytes == 0
        || corpus.input_bytes != expected
    {
        bail!("admitted BAM corpus differs from the run-2002 closure");
    }
    Ok(())
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: String::new(),
        negative_exit_nonzero: true,
        metrics: [
            ("reference_bases", "/reference_bases", "bases"),
            ("covered_bases", "/covered_bases", "bases"),
            ("coverage_fraction", "/coverage_fraction", "fraction"),
            ("mean_depth", "/mean_depth", "depth"),
        ]
        .into_iter()
        .map(|(id, pointer, unit)| BenchmarkAcceptanceMetric {
            id: id.to_string(),
            path: "repetitions/repetition-1.metrics.json".to_string(),
            json_pointer: pointer.to_string(),
            operator: BenchmarkAcceptanceOperator::GreaterThan,
            expected: BenchmarkAcceptanceExpectedValue::Integer(0),
            unit: unit.to_string(),
        })
        .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::{AdmittedBamCorpus, AdmittedBamCorpusFile};

    use super::prepare_bam_coverage_material_from_corpus;

    #[test]
    fn material_closes_all_run_2002_tools() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let temporary = bijux_dna_infra::temp_dir("bam-coverage-material")?;
        let benchmark_root = temporary.path().join("benchmarks");
        let (registry, _, _) = crate::load_checked_in_benchmark_planning_contracts()?;
        let plan = crate::plan_stage_campaign_attempts(
            &registry,
            &"a".repeat(40),
            &"b".repeat(64),
            "run-2002",
            "horse-coverage",
            "attempt-001",
            "2026-08-05T00:00:00Z",
        )?;
        crate::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;
        let corpus = admitted_corpus(&repository, temporary.path())?;
        let source = source_contracts();
        let mut reference_span_digests = std::collections::BTreeSet::new();

        assert_eq!(plan.cycle.attempts.len(), 3);
        for attempt in &plan.cycle.attempts {
            let material = prepare_bam_coverage_material_from_corpus(
                &repository,
                &benchmark_root,
                &attempt.identity,
                &source,
                "operator",
                "2026-08-05T00:01:00Z",
                "project",
                "compute",
                &corpus,
            )?;
            assert!(material.execution.measured_command[2].contains("bam_coverage.awk"));
            assert_eq!(material.execution.parameters.effective_path, "effective-parameters.json");
            assert_eq!(material.execution.acceptance.metrics.len(), 4);
            let reference_span = material
                .execution
                .inputs
                .iter()
                .find(|input| input.id == "bam_header_reference_span")
                .expect("common reference-span input");
            assert_eq!(reference_span.mount_path, "/input/reference-span.tsv");
            reference_span_digests.insert(reference_span.sha256.clone());
            let expected_files = match attempt.identity.tool_id.as_str() {
                "mosdepth" => 30,
                "bedtools" | "samtools" => 18,
                _ => unreachable!(),
            };
            assert_eq!(material.execution.required_files.len(), expected_files);
        }
        assert_eq!(reference_span_digests.len(), 1);
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

    fn admitted_corpus(
        repository: &std::path::Path,
        root: &std::path::Path,
    ) -> anyhow::Result<AdmittedBamCorpus> {
        let bam = repository.join("assets/toy/core-v1/bam/validation_pass.bam");
        let bai = repository.join("assets/toy/core-v1/bam/validation_pass.bam.bai");
        let bam_size = std::fs::metadata(&bam)?.len();
        let bai_size = std::fs::metadata(&bai)?.len();
        Ok(AdmittedBamCorpus {
            corpus_id: "ena-prjeb22390-err2112767-bam".to_string(),
            sample_id: "ERR2112767".to_string(),
            data_index_path: root.join("data-index.json").display().to_string(),
            data_index_sha256: "a".repeat(64),
            catalog_path: root.join("catalog.json").display().to_string(),
            catalog_sha256: "b".repeat(64),
            bam: AdmittedBamCorpusFile {
                path: bam.display().to_string(),
                sha256: bijux_dna_infra::hash_file_sha256(&bam)?,
                size_bytes: bam_size,
            },
            bam_index: AdmittedBamCorpusFile {
                path: bai.display().to_string(),
                sha256: bijux_dna_infra::hash_file_sha256(&bai)?,
                size_bytes: bai_size,
            },
            input_bytes: bam_size + bai_size,
        })
    }
}
