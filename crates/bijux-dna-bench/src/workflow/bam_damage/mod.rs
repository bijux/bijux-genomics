//! Owner: bijux-dna-bench
//! Executable material for the governed BAM damage qualification campaign.

mod commands;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_attempt_identity, AdmittedBamCorpus, AdmittedReferenceBundle,
    BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator,
    BenchmarkAttemptIdentity, BenchmarkEffectiveParameterValue as EffectiveValue,
    BenchmarkExecutionAcceptance, BenchmarkExecutionDeterminism, BenchmarkExecutionInput,
    BenchmarkExecutionInputKind, BenchmarkLifecycleStatus,
};
use bijux_dna_core::contract::ArtifactRole;

use self::commands::{measured_command, negative_command, required_files, warmup_command};
use super::bam_observation::{ensure_parameter_evidence, BamParameterPolicy};
use super::{
    load_attempt_lifecycle, BenchmarkExecutionMaterial, BenchmarkStagePlanInputMaterial,
    BenchmarkStagePlanOutputMaterial, BenchmarkStagePlanUnitMaterial,
};

const RUN_ID: &str = "run-2010";
const STAGE_ID: &str = "bam.damage";
const CORPUS_ID: &str = "ena-prjeb22390-err2112767-ngsbriggs-input";
const REFERENCE_ID: &str = "equcab3-gcf-002863925.1";
const SAMPLE_ID: &str = "ERR2112767";
const BAM_MOUNT: &str = "/input/ERR2112767.sorted.bam";
const REFERENCE_MOUNT: &str = "/input/EquCab3.0.fna";

#[derive(Debug, Clone, PartialEq)]
pub struct BamDamageMaterial {
    pub execution: BenchmarkExecutionMaterial,
    pub admitted_corpus: AdmittedBamCorpus,
    pub admitted_reference: AdmittedReferenceBundle,
}

#[allow(clippy::too_many_arguments)]
/// Prepare one planned BAM-damage attempt from checksum-admitted evidence.
///
/// # Errors
/// Returns an error when identity, lifecycle, evidence, parameters, or tool binding differs from
/// the governed run-2010 qualification closure.
pub fn prepare_bam_damage_material_from_evidence(
    repository_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    corpus: &AdmittedBamCorpus,
    reference: &AdmittedReferenceBundle,
) -> Result<BamDamageMaterial> {
    validate_attempt_identity(identity).context("validate BAM damage attempt identity")?;
    if identity.run.run_id != RUN_ID
        || identity.run.domain != "bam"
        || identity.run.stage_id != STAGE_ID
        || !matches!(
            identity.tool_id.as_str(),
            "addeam" | "damageprofiler" | "mapdamage2" | "ngsbriggs" | "pmdtools" | "pydamage"
        )
    {
        bail!("attempt identity is outside the governed run-2010 closure");
    }
    if load_attempt_lifecycle(benchmark_root, identity)?.current_status()
        != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("BAM damage material requires a wholly planned attempt");
    }
    validate_evidence(corpus, reference)?;
    let attempt_directory = crate::benchmark_attempt_directory(benchmark_root, identity)?;
    let parameters = ensure_parameter_evidence(
        repository_root,
        &attempt_directory,
        SAMPLE_ID,
        BamParameterPolicy {
            stage_id: STAGE_ID,
            defaults_provider: "bijux_dna_domain_bam::stage_specs::stage_spec(BamStage::Damage)",
        },
        BTreeMap::from([
            ("udg_model".to_string(), EffectiveValue::Text("non_udg".to_string())),
            ("pmd_threshold_5p".to_string(), EffectiveValue::Number(0.3)),
            ("pmd_threshold_3p".to_string(), EffectiveValue::Number(0.3)),
            ("trim_5p".to_string(), EffectiveValue::Integer(2)),
            ("trim_3p".to_string(), EffectiveValue::Integer(2)),
            (
                "damage_tool_profile".to_string(),
                EffectiveValue::Text("ancient_dna_evidence".to_string()),
            ),
            ("evidence_only".to_string(), EffectiveValue::Boolean(true)),
        ]),
    )?;
    let input_bytes = corpus
        .input_bytes
        .checked_add(reference.input_bytes)
        .context("BAM damage input size overflow")?;
    let execution = BenchmarkExecutionMaterial {
        operator: operator.to_string(),
        prepared_at_utc: prepared_at_utc.to_string(),
        repository_path: repository_root.to_path_buf(),
        benchmark_root: benchmark_root.to_path_buf(),
        source_contract_sha256: source_contract_sha256.clone(),
        inputs: execution_inputs(corpus, reference),
        warmup_command: warmup_command(&identity.tool_id),
        measured_command: measured_command(
            &identity.tool_id,
            SAMPLE_ID,
            corpus.bam.size_bytes,
            corpus.bam_index.size_bytes,
            reference.input_bytes,
            input_bytes,
        ),
        negative_command: negative_command(&identity.tool_id),
        parameters,
        required_files: required_files(3),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: 16_384,
        walltime: "01:00:00".to_string(),
        scratch_mb: 8192,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: acceptance(),
        stage_plan_units: Some(vec![stage_plan_unit(&identity.tool_id)?]),
    };
    Ok(BamDamageMaterial {
        execution,
        admitted_corpus: corpus.clone(),
        admitted_reference: reference.clone(),
    })
}

fn validate_evidence(
    corpus: &AdmittedBamCorpus,
    reference: &AdmittedReferenceBundle,
) -> Result<()> {
    let bam_bytes = corpus
        .bam
        .size_bytes
        .checked_add(corpus.bam_index.size_bytes)
        .context("BAM damage corpus size overflow")?;
    let reference_bytes = reference
        .fasta
        .size_bytes
        .checked_add(reference.fasta_index.size_bytes)
        .and_then(|value| value.checked_add(reference.sequence_dictionary.size_bytes))
        .context("BAM damage reference size overflow")?;
    if corpus.corpus_id != CORPUS_ID
        || corpus.sample_id != SAMPLE_ID
        || reference.reference_id != REFERENCE_ID
        || corpus.input_bytes != bam_bytes
        || reference.input_bytes != reference_bytes
        || corpus.data_index_path != reference.data_index_path
        || corpus.data_index_sha256 != reference.data_index_sha256
        || [
            corpus.bam.size_bytes,
            corpus.bam_index.size_bytes,
            reference.fasta.size_bytes,
            reference.fasta_index.size_bytes,
            reference.sequence_dictionary.size_bytes,
        ]
        .contains(&0)
    {
        bail!("admitted evidence differs from the run-2010 closure");
    }
    Ok(())
}

fn execution_inputs(
    corpus: &AdmittedBamCorpus,
    reference: &AdmittedReferenceBundle,
) -> Vec<BenchmarkExecutionInput> {
    let mut inputs = vec![
        input(
            "bam",
            BenchmarkExecutionInputKind::Data,
            &corpus.bam.path,
            BAM_MOUNT,
            &corpus.bam.sha256,
        ),
        input(
            "bam_index",
            BenchmarkExecutionInputKind::Data,
            &corpus.bam_index.path,
            "/input/ERR2112767.sorted.bam.bai",
            &corpus.bam_index.sha256,
        ),
        input(
            "corpus_catalog",
            BenchmarkExecutionInputKind::Data,
            &corpus.catalog_path,
            "/input/corpus-catalog.json",
            &corpus.catalog_sha256,
        ),
        input(
            "data_index",
            BenchmarkExecutionInputKind::Data,
            &corpus.data_index_path,
            "/input/data-index.json",
            &corpus.data_index_sha256,
        ),
        input(
            "reference",
            BenchmarkExecutionInputKind::Reference,
            &reference.fasta.path,
            REFERENCE_MOUNT,
            &reference.fasta.sha256,
        ),
        input(
            "reference_catalog",
            BenchmarkExecutionInputKind::Reference,
            &reference.catalog_path,
            "/input/reference-catalog.json",
            &reference.catalog_sha256,
        ),
        input(
            "reference_index",
            BenchmarkExecutionInputKind::Reference,
            &reference.fasta_index.path,
            "/input/EquCab3.0.fna.fai",
            &reference.fasta_index.sha256,
        ),
        input(
            "reference_dictionary",
            BenchmarkExecutionInputKind::Reference,
            &reference.sequence_dictionary.path,
            "/input/EquCab3.0.dict",
            &reference.sequence_dictionary.sha256,
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

fn stage_plan_unit(tool_id: &str) -> Result<BenchmarkStagePlanUnitMaterial> {
    let mut inputs = vec![stage_input("bam", BAM_MOUNT, ArtifactRole::Bam)];
    if matches!(tool_id, "mapdamage2" | "ngsbriggs") {
        inputs.push(stage_input("reference", REFERENCE_MOUNT, ArtifactRole::Reference));
    }
    let outputs = match tool_id {
        "addeam" => vec![stage_output("damage_profiles", "damage-profiles", ArtifactRole::Report)],
        "damageprofiler" => vec![stage_output(
            "damageprofiler_reports",
            "damageprofiler-reports",
            ArtifactRole::Report,
        )],
        "mapdamage2" => {
            vec![stage_output("mapdamage2_reports", "mapdamage2-reports", ArtifactRole::Report)]
        }
        "ngsbriggs" => {
            vec![stage_output("damage_report", "damage.ngsbriggs.json", ArtifactRole::ReportJson)]
        }
        "pmdtools" => vec![
            stage_output("pmd_scores", "pmd-scores.tsv", ArtifactRole::Evidence),
            stage_output("pmd_statistics", "pmd-statistics.txt", ArtifactRole::Report),
        ],
        "pydamage" => {
            vec![stage_output("pydamage_results", "pydamage-results", ArtifactRole::Report)]
        }
        _ => bail!("unsupported BAM damage stage-plan tool"),
    };
    Ok(BenchmarkStagePlanUnitMaterial {
        execution_unit_id: SAMPLE_ID.to_ascii_lowercase(),
        inputs,
        outputs,
        scientific_defaults: BTreeMap::new(),
        input_derived: BTreeMap::new(),
        operator_overrides: BTreeMap::new(),
        immutable_parameters: BTreeSet::new(),
    })
}

fn stage_input(
    binding_name: &str,
    path: &str,
    role: ArtifactRole,
) -> BenchmarkStagePlanInputMaterial {
    BenchmarkStagePlanInputMaterial {
        binding_name: binding_name.to_string(),
        path: PathBuf::from(path),
        role,
    }
}

fn stage_output(
    binding_name: &str,
    relative_path: &str,
    role: ArtifactRole,
) -> BenchmarkStagePlanOutputMaterial {
    BenchmarkStagePlanOutputMaterial {
        binding_name: binding_name.to_string(),
        relative_path: relative_path.into(),
        role,
    }
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: String::new(),
        negative_exit_nonzero: true,
        metrics: [
            ("normalization_complete", "boolean"),
            ("native_file_count", "files"),
            ("native_output_bytes", "bytes"),
        ]
        .into_iter()
        .map(|(id, unit)| BenchmarkAcceptanceMetric {
            id: id.to_string(),
            path: "repetitions/repetition-1.metrics.json".to_string(),
            json_pointer: format!("/{id}"),
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

    use bijux_dna_bench_model::{AdmittedBamCorpusFile, AdmittedReferenceBundle};

    use super::{prepare_bam_damage_material_from_evidence, stage_plan_unit, AdmittedBamCorpus};

    #[test]
    fn material_closes_all_six_run_2010_tools() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let temporary = bijux_dna_infra::temp_dir("bam-damage-material")?;
        let benchmark_root = temporary.path().join("benchmarks");
        let (registry, _, _) = crate::load_checked_in_benchmark_planning_contracts()?;
        let plan = crate::plan_stage_campaign_attempts(
            &registry,
            &"a".repeat(40),
            &"b".repeat(64),
            "run-2010",
            "bam-damage",
            "attempt-001",
            "2026-08-14T00:00:00Z",
        )?;
        crate::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;
        let (corpus, reference) = admitted_evidence(temporary.path());
        let source = BTreeMap::from([
            ("campaign_registry".to_string(), "b".repeat(64)),
            ("container_catalog".to_string(), "c".repeat(64)),
            ("image_inventory".to_string(), "d".repeat(64)),
            ("stage_interface_registry".to_string(), "e".repeat(64)),
        ]);

        assert_eq!(plan.cycle.attempts.len(), 6);
        for attempt in &plan.cycle.attempts {
            let material = prepare_bam_damage_material_from_evidence(
                &repository,
                &benchmark_root,
                &attempt.identity,
                &source,
                "operator",
                "2026-08-14T00:01:00Z",
                "project",
                "compute",
                &corpus,
                &reference,
            )?;
            assert!(material.execution.measured_command[2].contains("bam_damage.awk"));
            assert_eq!(material.execution.required_files.len(), 21);
            assert_eq!(material.execution.acceptance.metrics.len(), 3);
            assert_eq!(material.execution.stage_plan_units.as_ref().map(Vec::len), Some(1));
        }
        Ok(())
    }

    #[test]
    fn material_rejects_evidence_from_different_data_indexes() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("bam-damage-index-authority")?;
        let (corpus, mut reference) = admitted_evidence(temporary.path());
        reference.data_index_sha256 = "f".repeat(64);
        assert!(super::validate_evidence(&corpus, &reference).is_err());
        Ok(())
    }

    #[test]
    fn stage_plan_units_match_every_canonical_damage_binding() -> anyhow::Result<()> {
        let bindings = bijux_dna_stage_contract::canonical_tool_bindings()?;
        for tool_id in
            ["addeam", "damageprofiler", "mapdamage2", "ngsbriggs", "pmdtools", "pydamage"]
        {
            let unit = stage_plan_unit(tool_id)?;
            let binding = bindings
                .binding(
                    &bijux_dna_core::ids::StageId::new(super::STAGE_ID),
                    &bijux_dna_core::ids::ToolId::new(tool_id),
                )
                .ok_or_else(|| anyhow::anyhow!("canonical BAM damage binding is absent"))?;
            assert_eq!(
                unit.inputs.iter().map(|input| input.binding_name.as_str()).collect::<Vec<_>>(),
                binding.inputs.iter().map(|input| input.name.as_str()).collect::<Vec<_>>()
            );
            assert_eq!(
                unit.outputs.iter().map(|output| output.binding_name.as_str()).collect::<Vec<_>>(),
                binding.outputs.iter().map(|output| output.name.as_str()).collect::<Vec<_>>()
            );
        }
        assert!(stage_plan_unit("unknown").is_err());
        Ok(())
    }

    fn admitted_evidence(root: &std::path::Path) -> (AdmittedBamCorpus, AdmittedReferenceBundle) {
        let data_index_path = root.join("data-index.json").display().to_string();
        let corpus = AdmittedBamCorpus {
            corpus_id: super::CORPUS_ID.to_string(),
            sample_id: super::SAMPLE_ID.to_string(),
            data_index_path: data_index_path.clone(),
            data_index_sha256: "a".repeat(64),
            catalog_path: root.join("corpus.json").display().to_string(),
            catalog_sha256: "b".repeat(64),
            bam: file(root, "sample.bam", 100, 'c'),
            bam_index: file(root, "sample.bam.bai", 10, 'd'),
            input_bytes: 110,
        };
        let reference = AdmittedReferenceBundle {
            reference_id: super::REFERENCE_ID.to_string(),
            data_index_path,
            data_index_sha256: "a".repeat(64),
            catalog_path: root.join("reference.json").display().to_string(),
            catalog_sha256: "e".repeat(64),
            fasta: file(root, "reference.fasta", 200, 'f'),
            fasta_index: file(root, "reference.fasta.fai", 20, '1'),
            sequence_dictionary: file(root, "reference.dict", 30, '2'),
            input_bytes: 250,
        };
        (corpus, reference)
    }

    fn file(
        root: &std::path::Path,
        name: &str,
        size_bytes: u64,
        digest: char,
    ) -> AdmittedBamCorpusFile {
        AdmittedBamCorpusFile {
            path: root.join(name).display().to_string(),
            sha256: digest.to_string().repeat(64),
            size_bytes,
        }
    }
}
