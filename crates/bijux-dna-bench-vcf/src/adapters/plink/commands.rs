//! Owner: bijux-dna-bench-vcf
//! plink adapter commands contracts.
use super::{
    anyhow, governed_inputs_for_stage, load_governed_vcf_fixture_inputs, run_missing_input_probe,
    validate_command_steps, validate_required_inputs, vcf_domain_stage_expected_output_ids, Path,
    PathBuf, RegistryToolContract, Result, VcfDomainStage, VcfPlinkFamilyAdapterArtifact,
    VcfPlinkFamilyAdapterCommandStep, VcfPlinkFamilyAdapterRow, VcfStageCatalogRow,
    VcfStageMatrixRow, VCF_PLINK2_ADAPTER_SCHEMA_VERSION, VCF_PLINK_ADAPTER_SCHEMA_VERSION,
};

pub(super) fn build_plink_family_row(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: Option<&VcfStageMatrixRow>,
) -> Result<VcfPlinkFamilyAdapterRow> {
    let stage = VcfDomainStage::try_from(stage_id)
        .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
    let identity = crate::inputs::resolve_adapter_binding_identity(
        stage,
        stage_id,
        &registry_tool.tool_id,
        matrix_row,
        "vcf_cohort",
    )?;
    let output_root = format!(
        "artifacts/benchmark-smoke/readiness/adapters/{}/{}",
        registry_tool.tool_id, stage_id
    );
    let output_prefix = format!("{output_root}/{}", stage_output_name_hint(stage));
    let fixture_inputs = load_governed_vcf_fixture_inputs(repo_root)?;
    let required_inputs = governed_inputs_for_stage(stage, &fixture_inputs.sample_metadata_path);
    validate_required_inputs(repo_root, stage_id, &required_inputs)?;
    let stage_output_ids = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| anyhow!("VCF {stage_id} is missing expected output ids"))?
        .iter()
        .map(|output| (*output).to_string())
        .collect::<Vec<_>>();
    let StageContract {
        command_contract_source,
        normalized_metrics_artifact_id,
        normalized_metrics_path,
        raw_output_ids,
        parser_output_ids,
        declared_outputs,
        command_steps,
    } = build_stage_contract(registry_tool.tool_id.as_str(), stage, &output_root, &output_prefix)?;
    let argv_validation_passed =
        validate_command_steps(registry_tool.tool_id.as_str(), stage_id, &command_steps).is_ok();
    let (
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
    ) = run_missing_input_probe(repo_root, stage_id, &required_inputs);
    let benchmark_status = if matrix_row.is_some() {
        "benchmark_ready".to_string()
    } else {
        "not_benchmark_ready".to_string()
    };
    let reason = match (registry_tool.tool_id.as_str(), stage) {
        ("plink2", VcfDomainStage::Admixture) => format!(
            "row `{stage_id}` / `{}` keeps the PLINK2 PCA-proxy raw outputs explicit and maps them to `{}` instead of pretending PLINK2 owns a native Q-matrix",
            registry_tool.tool_id, normalized_metrics_artifact_id
        ),
        ("plink", VcfDomainStage::Admixture) => format!(
            "row `{stage_id}` / `{}` keeps the PLINK cohort-preparation outputs explicit and maps them to `{}` for the downstream admixture report contract",
            registry_tool.tool_id, normalized_metrics_artifact_id
        ),
        _ => format!(
            "row `{stage_id}` / `{}` renders concrete cohort-analysis argv with {} declared raw output(s) and maps them to `{}`",
            registry_tool.tool_id,
            raw_output_ids.len(),
            normalized_metrics_artifact_id
        ),
    };

    Ok(VcfPlinkFamilyAdapterRow {
        tool_id: registry_tool.tool_id.clone(),
        tool_status: registry_tool.tool_status.clone(),
        stage_id: stage_id.to_string(),
        stage_status: catalog_row.support_status.clone(),
        benchmark_status,
        adapter_id: identity.adapter,
        parser_id: identity.parser,
        corpus_id: identity.corpus,
        asset_profile_id: identity.asset_profile,
        command_contract_source,
        output_root,
        output_prefix,
        normalized_metrics_artifact_id,
        normalized_metrics_path,
        stage_output_ids,
        raw_output_ids,
        parser_output_ids,
        required_inputs,
        declared_outputs,
        command_steps,
        argv_validation_passed,
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
        reason,
    })
}

pub(super) struct StageContract {
    pub(super) command_contract_source: String,
    pub(super) normalized_metrics_artifact_id: String,
    pub(super) normalized_metrics_path: String,
    pub(super) raw_output_ids: Vec<String>,
    pub(super) parser_output_ids: Vec<String>,
    pub(super) declared_outputs: Vec<VcfPlinkFamilyAdapterArtifact>,
    pub(super) command_steps: Vec<VcfPlinkFamilyAdapterCommandStep>,
}

pub(super) fn build_stage_contract(
    tool_id: &str,
    stage: VcfDomainStage,
    output_root: &str,
    output_prefix: &str,
) -> Result<StageContract> {
    let contract = match (tool_id, stage) {
        ("plink", VcfDomainStage::Qc) => super::stages::plink_qc::build(output_root, output_prefix),
        ("plink", VcfDomainStage::Admixture) => {
            super::stages::plink_admixture::build(output_root, output_prefix)
        }
        ("plink", VcfDomainStage::PopulationStructure) => {
            super::stages::plink_population_structure::build(output_root, output_prefix)
        }
        ("plink2", VcfDomainStage::Qc) => {
            super::stages::plink2_qc::build(output_root, output_prefix)
        }
        ("plink2", VcfDomainStage::Pca) => {
            super::stages::plink2_pca::build(output_root, output_prefix)
        }
        ("plink2", VcfDomainStage::Admixture) => {
            super::stages::plink2_admixture::build(output_root, output_prefix)
        }
        ("plink2", VcfDomainStage::PopulationStructure) => {
            super::stages::plink2_population_structure::build(output_root, output_prefix)
        }
        ("plink2", VcfDomainStage::Roh) => {
            super::stages::plink2_roh::build(output_root, output_prefix)
        }
        _ => {
            return Err(anyhow!(
                "VCF {tool_id} adapter does not govern stage `{}`",
                stage.as_str()
            ));
        }
    };
    Ok(contract)
}

pub(super) fn schema_version_for_tool(tool_id: &str) -> &'static str {
    match tool_id {
        "plink" => VCF_PLINK_ADAPTER_SCHEMA_VERSION,
        "plink2" => VCF_PLINK2_ADAPTER_SCHEMA_VERSION,
        other => panic!("unsupported VCF plink family schema tool `{other}`"),
    }
}

pub(super) fn stage_output_name_hint(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::Qc => "qc",
        VcfDomainStage::Pca => "pca",
        VcfDomainStage::Admixture => "admixture",
        VcfDomainStage::PopulationStructure => "population_structure",
        VcfDomainStage::Roh => "roh",
        other => panic!("unsupported VCF plink family stage `{}`", other.as_str()),
    }
}

pub(super) fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfPlinkFamilyAdapterArtifact {
    VcfPlinkFamilyAdapterArtifact {
        artifact_id: artifact_id.to_string(),
        role: role.to_string(),
        path: path.to_string(),
    }
}

pub(super) fn step(
    step_id: &str,
    step_kind: &str,
    argv: Vec<&str>,
    declared_output_artifact_ids: &[&str],
) -> VcfPlinkFamilyAdapterCommandStep {
    VcfPlinkFamilyAdapterCommandStep {
        step_id: step_id.to_string(),
        step_kind: step_kind.to_string(),
        argv: argv.into_iter().map(str::to_string).collect(),
        declared_output_artifact_ids: declared_output_artifact_ids
            .iter()
            .map(|artifact_id| (*artifact_id).to_string())
            .collect(),
    }
}

pub(super) fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

pub(super) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
