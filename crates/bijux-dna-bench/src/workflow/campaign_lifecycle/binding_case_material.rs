//! Owner: bijux-dna-bench
//! Binding-driven executable material for independently admitted benchmark cases.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator,
    BenchmarkAttemptIdentity, BenchmarkCaseDefinitionV1, BenchmarkCaseMatrixV2,
    BenchmarkDataArtifactSnapshotV1, BenchmarkDataSnapshotRoleV1, BenchmarkEffectiveParameters,
    BenchmarkExecutionAcceptance, BenchmarkExecutionDeterminism, BenchmarkExecutionInput,
    BenchmarkExecutionInputKind, BenchmarkExecutionParameterEvidence,
    BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
};
use bijux_dna_core::contract::{
    canonical_artifact_contract_catalog, ArtifactClassV1, ArtifactFormatV1, ArtifactRole,
};
use bijux_dna_runtime::benchmark_attempt_directory;
use bijux_dna_stage_contract::{
    project_tool_parameters, render_tool_command, BindingInputSourceV1, ToolBinding,
    ToolBindingRenderContext, ToolParameterProjectionRequest,
};

use super::{
    BenchmarkExecutionMaterial, BenchmarkStagePlanInputMaterial, BenchmarkStagePlanOutputMaterial,
    BenchmarkStagePlanUnitMaterial,
};
use crate::workflow::parameter_authority::load_stage_parameter_authority;

/// Construct one case directly from its immutable matrix and canonical tool binding.
///
/// The first executable generic closure deliberately accepts only independently runnable,
/// single-input corpus FASTQ bindings. Other shapes return a case-local planning error instead of
/// blocking unrelated campaign units or silently guessing reference/upstream relationships.
#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_binding_case_material(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    source_contract_sha256: &BTreeMap<String, String>,
    operator: &str,
    prepared_at_utc: &str,
    account: &str,
    partition: &str,
    executable: &str,
    matrix: &BenchmarkCaseMatrixV2,
) -> Result<BenchmarkExecutionMaterial> {
    let case_id = identity
        .case_id
        .as_deref()
        .context("binding-driven benchmark attempt has no case identity")?;
    let case = matrix
        .cases
        .iter()
        .find(|case| case.case_id == case_id)
        .context("benchmark matrix omits the planned case identity")?;
    validate_case_identity(case, identity)?;
    if case.campaign.material_driver.is_some() {
        bail!("binding-driven material cannot replace a reviewed campaign material driver");
    }
    if !case.binding.parameters().is_empty() {
        bail!("binding-driven case has unresolved tool parameters");
    }
    let input = require_independent_corpus_fastq_binding(&case.binding)?;

    let snapshot = matrix
        .data_snapshots
        .iter()
        .find(|snapshot| {
            snapshot.role == BenchmarkDataSnapshotRoleV1::Corpus
                && snapshot.catalog_id == case.corpus.corpus_id
        })
        .context("benchmark matrix omits the case corpus snapshot")?;
    if snapshot.catalog_sha256 != case.corpus.snapshot_sha256
        || snapshot.sample_ids != case.corpus.sample_ids
    {
        bail!("benchmark case corpus differs from its matrix snapshot authority");
    }

    let output_materials = binding_outputs(case)?;
    let acceptance_output = json_acceptance_output(&output_materials)?;
    let acceptance_output_name = acceptance_output.relative_path.clone();
    let mut execution_inputs = Vec::with_capacity(case.corpus.sample_ids.len());
    let mut units = Vec::with_capacity(case.corpus.sample_ids.len());
    for sample_id in &case.corpus.sample_ids {
        let file = select_single_end_fastq(&snapshot.files, sample_id)?;
        let path = resolve_snapshot_file(study_root, file)?;
        let unit_id = canonical_component(sample_id)?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .context("benchmark FASTQ input has no UTF-8 file name")?;
        let mount_path = PathBuf::from("/input/corpus").join(file_name);
        execution_inputs.push(BenchmarkExecutionInput {
            id: format!("corpus-{unit_id}"),
            kind: BenchmarkExecutionInputKind::Data,
            path: path.display().to_string(),
            mount_path: mount_path.display().to_string(),
            sha256: file.sha256.clone(),
        });
        units.push(BenchmarkStagePlanUnitMaterial {
            execution_unit_id: unit_id,
            inputs: vec![BenchmarkStagePlanInputMaterial {
                binding_name: input.name.clone(),
                path: mount_path,
                role: ArtifactRole::Reads,
            }],
            outputs: output_materials.clone(),
            scientific_defaults: BTreeMap::new(),
            input_derived: BTreeMap::new(),
            operator_overrides: BTreeMap::new(),
            immutable_parameters: case.interface.immutable_parameters.iter().cloned().collect(),
        });
    }
    if execution_inputs.is_empty() {
        bail!("binding-driven case has no independently executable units");
    }

    let attempt_directory = benchmark_attempt_directory(benchmark_root, identity)?;
    let parameters = ensure_parameter_evidence(repository_root, &attempt_directory, case, &units)?;
    let warmup_command = render_warmup(case, executable, &units[0])?;
    let acceptance_path = format!(
        "repetitions/measured-repetition-1--{}/{}",
        units[0].execution_unit_id,
        acceptance_output_name.display()
    );
    let measured_path = format!(
        "/result/repetitions/measured-repetition-{{repetition}}--{}/{}",
        units[0].execution_unit_id,
        acceptance_output_name.display()
    );
    let resources = case.binding.resources();
    Ok(BenchmarkExecutionMaterial {
        operator: operator.to_string(),
        prepared_at_utc: prepared_at_utc.to_string(),
        repository_path: repository_root.to_path_buf(),
        benchmark_root: benchmark_root.to_path_buf(),
        source_contract_sha256: source_contract_sha256.clone(),
        inputs: execution_inputs,
        warmup_command,
        measured_command: vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            format!("test -s '{measured_path}'"),
        ],
        negative_command: vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            "test -r /input/intentionally-absent.fastq".to_string(),
        ],
        parameters,
        required_files: Vec::new(),
        account: account.to_string(),
        partition: partition.to_string(),
        memory_mb: u64::from(resources.mem_gb) * 1024,
        walltime: "02:00:00".to_string(),
        scratch_mb: u64::from(resources.tmp_gb) * 1024,
        random_seed: None,
        determinism: BenchmarkExecutionDeterminism::Deterministic,
        acceptance: BenchmarkExecutionAcceptance {
            version_regex: String::new(),
            negative_exit_nonzero: true,
            metrics: vec![BenchmarkAcceptanceMetric {
                id: "stage_identity".to_string(),
                path: acceptance_path,
                json_pointer: "/stage_id".to_string(),
                operator: BenchmarkAcceptanceOperator::Equal,
                expected: BenchmarkAcceptanceExpectedValue::Text(case.stage_id.clone()),
                unit: "identity".to_string(),
            }],
        },
        stage_plan_units: Some(units),
    })
}

fn json_acceptance_output(
    outputs: &[BenchmarkStagePlanOutputMaterial],
) -> Result<&BenchmarkStagePlanOutputMaterial> {
    outputs
        .iter()
        .find(|output| {
            matches!(
                output.role,
                ArtifactRole::ReportJson | ArtifactRole::MetricsJson | ArtifactRole::SummaryJson
            )
        })
        .context("binding-driven case requires a JSON output until its normalizer is executable")
}

fn require_independent_corpus_fastq_binding(
    binding: &bijux_dna_stage_contract::ToolBindingV1,
) -> Result<&bijux_dna_stage_contract::BindingInputV1> {
    let [input] = binding.inputs() else {
        bail!("binding-driven case requires exactly one declared input");
    };
    if !input.required
        || input.source != BindingInputSourceV1::Corpus
        || input.name != "reads_r1"
        || !input.artifact_contract_id.starts_with("bijux.fastq.artifact.")
    {
        bail!("binding-driven case input is not an independent corpus FASTQ contract");
    }
    Ok(input)
}

fn validate_case_identity(
    case: &BenchmarkCaseDefinitionV1,
    identity: &BenchmarkAttemptIdentity,
) -> Result<()> {
    if case.run_id != identity.run.run_id
        || case.domain != identity.run.domain
        || case.stage_id != identity.run.stage_id
        || case.tool_id != identity.tool_id
        || case.binding.stage_id().as_str() != case.stage_id
        || case.binding.tool_id().as_str() != case.tool_id
    {
        bail!("benchmark case identity differs from the planned attempt");
    }
    Ok(())
}

fn select_single_end_fastq<'a>(
    files: &'a [BenchmarkDataArtifactSnapshotV1],
    sample_id: &str,
) -> Result<&'a BenchmarkDataArtifactSnapshotV1> {
    let marker = format!("/{sample_id}/");
    let mut candidates = files
        .iter()
        .filter(|file| file.path.contains(&marker) && file.role == "raw-fastq")
        .filter(|file| {
            let name = file.path.rsplit('/').next().unwrap_or_default();
            name == format!("{sample_id}.fastq.gz")
                || name == format!("{sample_id}.fq.gz")
                || name == format!("{sample_id}_1.fastq.gz")
                || name == format!("{sample_id}_1.fq.gz")
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.path.cmp(&right.path));
    let [file] = candidates.as_slice() else {
        bail!("benchmark sample {sample_id} does not resolve to one R1 or single-end FASTQ");
    };
    Ok(file)
}

fn resolve_snapshot_file(
    study_root: &Path,
    file: &BenchmarkDataArtifactSnapshotV1,
) -> Result<PathBuf> {
    let relative =
        file.path.strip_prefix("study://").context("benchmark snapshot path is not a study URI")?;
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(component, Component::ParentDir | Component::CurDir | Component::RootDir)
        })
    {
        bail!("benchmark snapshot path escapes the governed STUDY root");
    }
    let path = study_root.join(relative);
    let metadata = std::fs::symlink_metadata(&path)
        .with_context(|| format!("inspect benchmark snapshot file {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() != file.size_bytes
    {
        bail!("benchmark snapshot file type or size differs from matrix authority");
    }
    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalize benchmark snapshot file {}", path.display()))?;
    if !canonical.starts_with(study_root)
        || bijux_dna_infra::hash_file_sha256(&canonical)? != file.sha256
    {
        bail!("benchmark snapshot file identity differs from matrix authority");
    }
    Ok(canonical)
}

fn binding_outputs(
    case: &BenchmarkCaseDefinitionV1,
) -> Result<Vec<BenchmarkStagePlanOutputMaterial>> {
    case.binding
        .outputs()
        .iter()
        .map(|output| {
            let (role, suffix) = artifact_role_and_suffix(&output.artifact_contract_id)?;
            Ok(BenchmarkStagePlanOutputMaterial {
                binding_name: output.name.clone(),
                relative_path: PathBuf::from(format!("{}{suffix}", output.name)),
                role,
            })
        })
        .collect()
}

fn artifact_role_and_suffix(contract_id: &str) -> Result<(ArtifactRole, &'static str)> {
    let catalog = canonical_artifact_contract_catalog();
    let contract = catalog
        .contract(contract_id)
        .with_context(|| format!("binding uses unknown artifact contract {contract_id}"))?;
    let format = contract.formats.first().context("artifact contract has no format")?;
    let value = match (contract.class, format) {
        (ArtifactClassV1::Fastq, _) => (ArtifactRole::Reads, ".fastq.gz"),
        (ArtifactClassV1::Alignment, ArtifactFormatV1::Sam) => (ArtifactRole::Alignment, ".sam"),
        (ArtifactClassV1::Alignment, _) => (ArtifactRole::Bam, ".bam"),
        (ArtifactClassV1::Variant, _) => (ArtifactRole::Variant, ".vcf.gz"),
        (ArtifactClassV1::Reference, _) => (ArtifactRole::Reference, ".fasta"),
        (ArtifactClassV1::Auxiliary, ArtifactFormatV1::Json) => (ArtifactRole::ReportJson, ".json"),
        (ArtifactClassV1::Auxiliary, ArtifactFormatV1::Tabular) => {
            (ArtifactRole::SummaryTsv, ".tsv")
        }
        (ArtifactClassV1::Auxiliary, ArtifactFormatV1::Html) => (ArtifactRole::ReportHtml, ".html"),
        (ArtifactClassV1::Auxiliary, ArtifactFormatV1::Index) => (ArtifactRole::Index, ".idx"),
        (ArtifactClassV1::Auxiliary, ArtifactFormatV1::Text) => (ArtifactRole::Log, ".txt"),
        (ArtifactClassV1::Auxiliary, ArtifactFormatV1::Directory) => (ArtifactRole::Evidence, ""),
        _ => (ArtifactRole::Evidence, ".artifact"),
    };
    Ok(value)
}

fn ensure_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    case: &BenchmarkCaseDefinitionV1,
    units: &[BenchmarkStagePlanUnitMaterial],
) -> Result<BenchmarkExecutionParameterEvidence> {
    let authority = load_stage_parameter_authority(repository_root, &case.stage_id)?;
    if !authority.stage.overrides.is_empty()
        || authority.stage.parameter_schema_version != case.interface.parameter_schema_version
        || authority.stage.defaults_provider != case.interface.default_provider
    {
        bail!("binding-driven case parameter authority requires an explicit projection");
    }
    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &authority.source_bytes)?;
    let effective = BenchmarkEffectiveParameters {
        schema_version: BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1.to_string(),
        stage_id: case.stage_id.clone(),
        parameter_schema_version: authority.stage.parameter_schema_version.clone(),
        defaults_provider: authority.stage.defaults_provider.clone(),
        overrides: BTreeMap::new(),
        resolved_by_sample: units
            .iter()
            .map(|unit| (unit.execution_unit_id.clone(), BTreeMap::new()))
            .collect(),
    };
    let mut effective_bytes = serde_json::to_vec_pretty(&effective)?;
    effective_bytes.push(b'\n');
    let effective_path = attempt_directory.join("effective-parameters.json");
    ensure_bytes(&effective_path, &effective_bytes)?;
    Ok(BenchmarkExecutionParameterEvidence {
        parameter_schema_version: authority.stage.parameter_schema_version,
        defaults_provider: authority.stage.defaults_provider,
        overrides_path: "config-pipeline-parameters.yml".to_string(),
        overrides_sha256: bijux_dna_infra::hash_bytes_sha256(&authority.source_bytes),
        effective_path: "effective-parameters.json".to_string(),
        effective_sha256: bijux_dna_infra::hash_bytes_sha256(&effective_bytes),
    })
}

fn render_warmup(
    case: &BenchmarkCaseDefinitionV1,
    executable: &str,
    unit: &BenchmarkStagePlanUnitMaterial,
) -> Result<Vec<String>> {
    let effective = project_tool_parameters(&ToolParameterProjectionRequest {
        binding: &case.binding,
        scientific_defaults: &unit.scientific_defaults,
        input_derived: &unit.input_derived,
        operator_overrides: &unit.operator_overrides,
        immutable_parameters: &unit.immutable_parameters,
        allocated_cpus: case.measurement.allocated_cpus,
    })?;
    let inputs = unit
        .inputs
        .iter()
        .map(|input| (input.binding_name.clone(), input.path.display().to_string()))
        .collect();
    let outputs = unit
        .outputs
        .iter()
        .map(|output| {
            (
                output.binding_name.clone(),
                Path::new("/scratch").join(&output.relative_path).display().to_string(),
            )
        })
        .collect();
    render_tool_command(
        &case.binding,
        &ToolBindingRenderContext {
            executable: executable.to_string(),
            inputs,
            input_sets: BTreeMap::new(),
            parameters: effective.render_values(),
            outputs,
        },
    )
}

fn canonical_component(value: &str) -> Result<String> {
    let canonical = value.to_ascii_lowercase().replace('_', "-");
    if canonical.is_empty()
        || !canonical
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        bail!("benchmark sample identity cannot form a canonical execution unit");
    }
    Ok(canonical)
}

fn ensure_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)? != bytes
            {
                bail!("binding-driven parameter evidence conflicts with immutable bytes");
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, bytes)?;
            Ok(())
        }
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        artifact_role_and_suffix, canonical_component, json_acceptance_output,
        require_independent_corpus_fastq_binding, select_single_end_fastq,
    };
    use bijux_dna_bench_model::BenchmarkDataArtifactSnapshotV1;
    use bijux_dna_core::contract::ArtifactRole;
    use bijux_dna_core::ids::{StageId, ToolId};
    use bijux_dna_stage_contract::canonical_tool_bindings;

    #[test]
    fn generic_fastq_selection_uses_r1_or_single_end_without_admitting_r2() -> anyhow::Result<()> {
        let file = |name: &str| BenchmarkDataArtifactSnapshotV1 {
            path: format!("study://data/corpus/SAMPLE/{name}"),
            sha256: "a".repeat(64),
            size_bytes: 1,
            role: "raw-fastq".to_string(),
        };
        let paired = vec![file("SAMPLE_2.fastq.gz"), file("SAMPLE_1.fastq.gz")];
        assert!(select_single_end_fastq(&paired, "SAMPLE")?.path.ends_with("_1.fastq.gz"));
        let single = vec![file("SAMPLE.fastq.gz")];
        assert!(select_single_end_fastq(&single, "SAMPLE")?.path.ends_with("SAMPLE.fastq.gz"));
        Ok(())
    }

    #[test]
    fn generic_output_shape_comes_from_the_artifact_contract() -> anyhow::Result<()> {
        assert_eq!(
            artifact_role_and_suffix("bijux.auxiliary.artifact.json.v1")?,
            (ArtifactRole::ReportJson, ".json")
        );
        assert_eq!(canonical_component("ERR_42")?, "err-42");
        Ok(())
    }

    #[test]
    fn driverless_fastq_observation_bindings_enter_the_generic_case_path() -> anyhow::Result<()> {
        let bindings = canonical_tool_bindings()?;
        for stage_id in
            ["fastq.detect_duplicates_premerge", "fastq.estimate_library_complexity_prealign"]
        {
            let binding = bindings
                .binding(&StageId::new(stage_id), &ToolId::new("bijux_dna"))
                .ok_or_else(|| anyhow::anyhow!("missing canonical binding for {stage_id}"))?;
            assert!(binding.parameters.is_empty());
            assert_eq!(require_independent_corpus_fastq_binding(binding)?.name, "reads_r1");
        }
        Ok(())
    }

    #[test]
    fn non_json_binding_outputs_fail_before_generic_execution() -> anyhow::Result<()> {
        let bindings = canonical_tool_bindings()?;
        let binding = bindings
            .binding(&StageId::new("fastq.filter_reads"), &ToolId::new("bbduk"))
            .ok_or_else(|| anyhow::anyhow!("missing canonical filter binding"))?;
        let outputs = binding
            .outputs
            .iter()
            .map(|output| {
                let (role, suffix) = artifact_role_and_suffix(&output.artifact_contract_id)?;
                Ok(super::BenchmarkStagePlanOutputMaterial {
                    binding_name: output.name.clone(),
                    relative_path: std::path::PathBuf::from(format!("{}{suffix}", output.name)),
                    role,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        assert!(json_acceptance_output(&outputs).is_err());
        Ok(())
    }
}
