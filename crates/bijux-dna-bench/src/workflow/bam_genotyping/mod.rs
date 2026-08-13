//! Owner: bijux-dna-bench
//! Executable material for the governed BAM genotyping campaign.

mod commands;
mod report;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric,
    BenchmarkAcceptanceOperator, BenchmarkAttemptIdentity,
    BenchmarkEffectiveParameterValue as EffectiveValue, BenchmarkExecutionAcceptance,
    BenchmarkExecutionDeterminism, BenchmarkExecutionInput, BenchmarkExecutionInputKind,
    BenchmarkLifecycleStatus,
};

use self::commands::{measured_command, negative_command, required_files, warmup_command};
use super::bam_observation::{ensure_parameter_evidence, BamParameterPolicy};
use super::{load_attempt_lifecycle, BenchmarkExecutionMaterial};
use crate::AdmittedBamGenotypingCorpus;

pub(crate) use self::report::publish_bam_genotyping_stage_report_with_presentation_authority;
pub use self::report::{
    publish_bam_genotyping_stage_report, render_bam_genotyping_markdown,
    BamGenotypingStageReportPublication,
};

const RUN_ID: &str = "run-2001";
const STAGE_ID: &str = "bam.genotyping";
const SAMPLE_ID: &str = "ERR2112767";

#[derive(Debug, Clone, PartialEq)]
pub struct BamGenotypingMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub admitted_corpus: AdmittedBamGenotypingCorpus,
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_bam_genotyping_material(
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
) -> Result<BamGenotypingMaterial> {
    let corpus = crate::admit_bam_genotyping_corpus(study_root, data_index_path)?;
    prepare_bam_genotyping_material_from_corpus(
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
pub fn prepare_bam_genotyping_material_from_corpus(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    corpus: &AdmittedBamGenotypingCorpus,
) -> Result<BamGenotypingMaterial> {
    validate_attempt_identity(identity).context("validate BAM genotyping attempt")?;
    if identity.run.run_id != RUN_ID
        || identity.run.domain != "bam"
        || identity.run.stage_id != STAGE_ID
        || identity.tool_id != "bcftools"
    {
        bail!("attempt identity is outside the governed run-2001 closure");
    }
    if load_attempt_lifecycle(benchmark_root, identity)?.current_status()
        != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("BAM genotyping material requires a wholly planned attempt");
    }
    if corpus.bam.sample_id != SAMPLE_ID
        || corpus.bam_input_bytes == 0
        || corpus.reference_input_bytes == 0
        || corpus.database_input_bytes == 0
        || corpus.input_bytes
            != corpus
                .bam_input_bytes
                .checked_add(corpus.reference_input_bytes)
                .and_then(|value| value.checked_add(corpus.database_input_bytes))
                .context("genotyping input size overflow")?
    {
        bail!("admitted BAM genotyping corpus differs from the run-2001 closure");
    }
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let parameters = ensure_parameter_evidence(
        repository_root,
        &attempt_directory,
        SAMPLE_ID,
        BamParameterPolicy {
            stage_id: STAGE_ID,
            defaults_provider:
                "bijux_dna_domain_bam::stage_specs::stage_spec(BamStage::Genotyping)",
        },
        BTreeMap::from([
            ("caller".to_string(), EffectiveValue::Text("bcftools".to_string())),
            ("min_posterior".to_string(), EffectiveValue::Number(0.8)),
            ("min_call_rate".to_string(), EffectiveValue::Number(0.7)),
        ]),
    )?;
    let execution = BenchmarkExecutionMaterial {
        operator: operator.to_string(),
        prepared_at_utc: prepared_at_utc.to_string(),
        repository_path: PathBuf::from(repository_root),
        benchmark_root: PathBuf::from(benchmark_root),
        source_contract_sha256: source_contract_sha256.clone(),
        inputs: execution_inputs(corpus),
        warmup_command: warmup_command(),
        measured_command: measured_command(
            SAMPLE_ID,
            corpus.bam_input_bytes,
            corpus.reference_input_bytes,
            corpus.database_input_bytes,
            corpus.input_bytes,
        ),
        negative_command: negative_command(),
        parameters,
        required_files: required_files(3),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: 8192,
        walltime: "00:30:00".to_string(),
        scratch_mb: 4096,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
        stage_plan_units: None,
    };
    Ok(BamGenotypingMaterial { execution, admitted_corpus: corpus.clone() })
}

fn execution_inputs(corpus: &AdmittedBamGenotypingCorpus) -> Vec<BenchmarkExecutionInput> {
    let mut inputs = vec![
        input(
            "bam",
            BenchmarkExecutionInputKind::Data,
            &corpus.bam.bam.path,
            "/input/ERR2112767.sorted.bam",
            &corpus.bam.bam.sha256,
        ),
        input(
            "bam_index",
            BenchmarkExecutionInputKind::Data,
            &corpus.bam.bam_index.path,
            "/input/ERR2112767.sorted.bam.bai",
            &corpus.bam.bam_index.sha256,
        ),
        input(
            "corpus_data_index",
            BenchmarkExecutionInputKind::Data,
            &corpus.bam.data_index_path,
            "/input/data-index.json",
            &corpus.bam.data_index_sha256,
        ),
        input(
            "bam_catalog",
            BenchmarkExecutionInputKind::Data,
            &corpus.bam.catalog_path,
            "/input/ERR2112767.catalog.json",
            &corpus.bam.catalog_sha256,
        ),
        input(
            "reference",
            BenchmarkExecutionInputKind::Reference,
            &corpus.reference.path,
            "/input/EquCab3.0.fna",
            &corpus.reference.sha256,
        ),
        input(
            "reference_index",
            BenchmarkExecutionInputKind::Reference,
            &corpus.reference_index.path,
            "/input/EquCab3.0.fna.fai",
            &corpus.reference_index.sha256,
        ),
        input(
            "reference_catalog",
            BenchmarkExecutionInputKind::Reference,
            &corpus.reference_catalog.path,
            "/input/EquCab3.0.catalog.json",
            &corpus.reference_catalog.sha256,
        ),
        input(
            "known_sites",
            BenchmarkExecutionInputKind::Database,
            &corpus.known_sites.path,
            "/input/known-sites.vcf.gz",
            &corpus.known_sites.sha256,
        ),
        input(
            "known_sites_index",
            BenchmarkExecutionInputKind::Database,
            &corpus.known_sites_index.path,
            "/input/known-sites.vcf.gz.csi",
            &corpus.known_sites_index.sha256,
        ),
        input(
            "candidate_regions",
            BenchmarkExecutionInputKind::Database,
            &corpus.candidate_regions.path,
            "/input/known-sites.bed",
            &corpus.candidate_regions.sha256,
        ),
        input(
            "known_sites_catalog",
            BenchmarkExecutionInputKind::Database,
            &corpus.known_sites_catalog.path,
            "/input/known-sites.catalog.json",
            &corpus.known_sites_catalog.sha256,
        ),
        input(
            "constrained_alleles",
            BenchmarkExecutionInputKind::Database,
            &corpus.constrained_alleles.path,
            "/input/known-sites.alleles.tsv",
            &corpus.constrained_alleles.sha256,
        ),
        input(
            "genotyping_inputs_catalog",
            BenchmarkExecutionInputKind::Database,
            &corpus.genotyping_inputs_catalog.path,
            "/input/genotyping-inputs.catalog.json",
            &corpus.genotyping_inputs_catalog.sha256,
        ),
    ];
    inputs.sort_by(|left, right| left.id.cmp(&right.id));
    inputs
}

fn input(
    id: &str,
    kind: BenchmarkExecutionInputKind,
    path: &str,
    mount_path: &str,
    sha256: &str,
) -> BenchmarkExecutionInput {
    BenchmarkExecutionInput {
        id: id.to_string(),
        kind,
        path: path.to_string(),
        mount_path: mount_path.to_string(),
        sha256: sha256.to_string(),
    }
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: "^bcftools 1[.]20".to_string(),
        negative_exit_nonzero: true,
        metrics: ["total_records", "covered_sites", "called_genotypes"]
            .into_iter()
            .map(|id| BenchmarkAcceptanceMetric {
                id: id.to_string(),
                path: "repetitions/repetition-1.metrics.json".to_string(),
                json_pointer: format!("/{id}"),
                operator: BenchmarkAcceptanceOperator::GreaterThan,
                expected: BenchmarkAcceptanceExpectedValue::Integer(0),
                unit: "sites".to_string(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::{AdmittedBamCorpus, AdmittedBamCorpusFile};

    use super::{prepare_bam_genotyping_material_from_corpus, AdmittedBamGenotypingCorpus};

    #[test]
    fn material_closes_the_run_2001_attempt() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let temporary = bijux_dna_infra::temp_dir("bam-genotyping-material")?;
        let benchmark_root = temporary.path().join("benchmarks");
        let (registry, _, _) = crate::load_checked_in_benchmark_planning_contracts()?;
        let plan = crate::plan_stage_campaign_attempts(
            &registry,
            &"a".repeat(40),
            &"b".repeat(64),
            "run-2001",
            "horse-genotyping",
            "attempt-001",
            "2026-08-05T00:00:00Z",
        )?;
        crate::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;
        let material = prepare_bam_genotyping_material_from_corpus(
            &repository,
            &benchmark_root,
            &plan.cycle.attempts[0].identity,
            &source_contracts(),
            "operator",
            "2026-08-05T00:01:00Z",
            "project",
            "compute",
            &admitted_corpus(temporary.path()),
        )?;
        assert_eq!(material.execution.inputs.len(), 13);
        assert!(material.execution.inputs.iter().any(|input| {
            input.id == "corpus_data_index" && input.mount_path == "/input/data-index.json"
        }));
        assert_eq!(material.execution.required_files.len(), 27);
        assert!(material.execution.measured_command[2].contains("bcftools call"));
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

    fn admitted_corpus(root: &std::path::Path) -> AdmittedBamGenotypingCorpus {
        let file = |name: &str, size_bytes| AdmittedBamCorpusFile {
            path: root.join(name).display().to_string(),
            sha256: "f".repeat(64),
            size_bytes,
        };
        AdmittedBamGenotypingCorpus {
            bam: AdmittedBamCorpus {
                corpus_id: "ena-prjeb22390-err2112767-bam".to_string(),
                sample_id: "ERR2112767".to_string(),
                data_index_path: root.join("data-index.json").display().to_string(),
                data_index_sha256: "a".repeat(64),
                catalog_path: root.join("bam-catalog.json").display().to_string(),
                catalog_sha256: "b".repeat(64),
                bam: file("ERR2112767.sorted.bam", 23),
                bam_index: file("ERR2112767.sorted.bam.bai", 3),
                input_bytes: 26,
            },
            reference: file("EquCab3.0.fna", 70),
            reference_index: file("EquCab3.0.fna.fai", 7),
            reference_catalog: file("reference-catalog.json", 2),
            known_sites: file("known-sites.vcf.gz", 40),
            known_sites_index: file("known-sites.vcf.gz.csi", 4),
            candidate_regions: file("known-sites.bed", 10),
            known_sites_catalog: file("known-sites-catalog.json", 2),
            constrained_alleles: file("known-sites.alleles.tsv", 20),
            genotyping_inputs_catalog: file("genotyping-inputs-catalog.json", 2),
            bam_input_bytes: 26,
            reference_input_bytes: 77,
            database_input_bytes: 74,
            input_bytes: 177,
        }
    }
}
