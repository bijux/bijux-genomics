//! Owner: bijux-dna-bench-vcf
//! descent adapter commands contracts.
use super::{
    anyhow, bail, run_missing_input_probe, validate_command_steps, validate_required_inputs,
    vcf_domain_stage_expected_output_ids, Path, PathBuf, RegistryToolContract, Result,
    VcfDescentFamilyAdapterArtifact, VcfDescentFamilyAdapterCommandStep,
    VcfDescentFamilyAdapterRow, VcfDomainStage, VcfStageCatalogRow, VcfStageMatrixRow,
};

pub(super) fn build_descent_family_row(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: Option<&VcfStageMatrixRow>,
) -> Result<VcfDescentFamilyAdapterRow> {
    let stage = VcfDomainStage::try_from(stage_id)
        .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
    let identity = crate::inputs::resolve_adapter_binding_identity(
        stage,
        stage_id,
        &registry_tool.tool_id,
        matrix_row,
        if stage == VcfDomainStage::Demography { "json_ibd_segments" } else { "vcf_cohort" },
    )?;
    let output_root = format!(
        "artifacts/benchmark-smoke/readiness/adapters/descent/{}/{}",
        registry_tool.tool_id, stage_id
    );
    let StageContract {
        input_vcf_path,
        input_ibd_segments_path,
        required_inputs,
        command_contract_source,
        normalized_output_artifact_id,
        normalized_output_path,
        log_output_path,
        raw_output_ids,
        parser_output_ids,
        declared_outputs,
        command_steps,
    } = build_stage_contract(repo_root, registry_tool.tool_id.as_str(), stage, &output_root)?;
    validate_required_inputs(repo_root, stage_id, &required_inputs)?;
    let stage_output_ids = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| anyhow!("VCF {stage_id} is missing expected output ids"))?
        .iter()
        .map(|output| (*output).to_string())
        .collect::<Vec<_>>();
    let argv_validation_passed =
        validate_command_steps(registry_tool.tool_id.as_str(), stage_id, &command_steps).is_ok();
    let (
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
    ) = run_missing_input_probe(stage_id, &required_inputs)?;
    let benchmark_status = if matrix_row.is_some() {
        "benchmark_ready".to_string()
    } else {
        "not_benchmark_ready".to_string()
    };
    let output_prefix = strip_known_suffix(&normalized_output_path);
    let reason = format!(
        "row `{stage_id}` / `{}` keeps `{}` explicit at `{}` and {}",
        registry_tool.tool_id,
        normalized_output_artifact_id,
        normalized_output_path,
        if benchmark_status == "benchmark_ready" {
            "remains admitted for benchmark job generation"
        } else {
            "stays visible as retained but not benchmark-ready"
        }
    );

    Ok(VcfDescentFamilyAdapterRow {
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
        input_vcf_path,
        input_ibd_segments_path,
        normalized_output_artifact_id,
        normalized_output_path,
        log_output_path,
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
    pub(super) input_vcf_path: Option<String>,
    pub(super) input_ibd_segments_path: Option<String>,
    pub(super) required_inputs: Vec<VcfDescentFamilyAdapterArtifact>,
    pub(super) command_contract_source: String,
    pub(super) normalized_output_artifact_id: String,
    pub(super) normalized_output_path: String,
    pub(super) log_output_path: String,
    pub(super) raw_output_ids: Vec<String>,
    pub(super) parser_output_ids: Vec<String>,
    pub(super) declared_outputs: Vec<VcfDescentFamilyAdapterArtifact>,
    pub(super) command_steps: Vec<VcfDescentFamilyAdapterCommandStep>,
}

pub(super) fn build_stage_contract(
    repo_root: &Path,
    tool_id: &str,
    stage: VcfDomainStage,
    output_root: &str,
) -> Result<StageContract> {
    match (tool_id, stage) {
        ("plink2", VcfDomainStage::Roh) => Ok(super::stages::plink2_roh::build(output_root)),
        ("germline", VcfDomainStage::Ibd) => Ok(super::stages::germline_ibd::build(output_root)),
        ("ibdseq", VcfDomainStage::Ibd) => Ok(super::stages::ibdseq_ibd::build(output_root)),
        ("ibdhap", VcfDomainStage::Ibd) => Ok(super::stages::ibdhap_ibd::build(output_root)),
        ("ibdne", VcfDomainStage::Demography) => {
            super::stages::ibdne_demography::build(repo_root, output_root)
        }
        _ => bail!("unsupported VCF descent adapter row `{}` / `{}`", tool_id, stage.as_str()),
    }
}

pub(super) fn artifact(
    artifact_id: &str,
    role: &str,
    path: &str,
) -> VcfDescentFamilyAdapterArtifact {
    VcfDescentFamilyAdapterArtifact {
        artifact_id: artifact_id.to_string(),
        role: role.to_string(),
        path: path.to_string(),
    }
}

pub(super) fn step(
    step_id: &str,
    step_kind: &str,
    argv: Vec<String>,
    declared_output_artifact_ids: &[&str],
) -> VcfDescentFamilyAdapterCommandStep {
    VcfDescentFamilyAdapterCommandStep {
        step_id: step_id.to_string(),
        step_kind: step_kind.to_string(),
        argv,
        declared_output_artifact_ids: declared_output_artifact_ids
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
    }
}

pub(super) fn strip_known_suffix(path: &str) -> String {
    for suffix in [".json", ".tsv", ".vcf.gz", ".vcf", ".log"] {
        if let Some(prefix) = path.strip_suffix(suffix) {
            return prefix.to_string();
        }
    }
    path.to_string()
}

pub(super) fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

pub(super) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
