//! Owner: bijux-dna-bench-vcf
//! angsd adapter commands contracts.
use super::{
    anyhow, governed_inputs_for_stage, materialize_bam_list, run_missing_input_probe,
    validate_command_steps, validate_required_inputs, vcf_domain_stage_adapter_id,
    vcf_domain_stage_parser_id, Path, PathBuf, Result, VcfAngsdAdapterArtifact,
    VcfAngsdAdapterCommandStep, VcfAngsdAdapterRow, VcfDomainStage, VcfStageCatalogRow,
    GOVERNED_ANGSD_TOOL_ID, GOVERNED_ANGSD_TOOL_STATUS, GOVERNED_BAM_PATH, GOVERNED_CORPUS_ID,
};

pub(super) fn build_angsd_row(
    repo_root: &Path,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    benchmark_ready: bool,
) -> Result<VcfAngsdAdapterRow> {
    let stage = VcfDomainStage::try_from(stage_id)
        .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
    let adapter_id = vcf_domain_stage_adapter_id(stage)
        .ok_or_else(|| anyhow!("VCF angsd adapter row `{stage_id}` is missing adapter id"))?;
    let parser_id = vcf_domain_stage_parser_id(stage)
        .ok_or_else(|| anyhow!("VCF angsd adapter row `{stage_id}` is missing parser id"))?;
    let output_root =
        format!("artifacts/benchmark-smoke/readiness/adapters/{GOVERNED_ANGSD_TOOL_ID}/{stage_id}");
    let bam_member_paths =
        if uses_bam_inputs(stage) { vec![GOVERNED_BAM_PATH.to_string()] } else { Vec::new() };
    let bam_list_path = if uses_bam_inputs(stage) {
        Some(materialize_bam_list(repo_root, &output_root, &bam_member_paths)?)
    } else {
        None
    };
    let required_inputs = governed_inputs_for_stage(stage, bam_list_path.as_deref())?;
    validate_required_inputs(repo_root, stage_id, &required_inputs)?;
    let StageContract {
        command_contract_source,
        reference_path,
        sites_path,
        likelihood_model,
        output_prefix,
        stage_output_ids,
        raw_output_ids,
        parser_output_ids,
        declared_outputs,
        command_steps,
    } = build_stage_adapter_contract(stage, &output_root, &required_inputs)?;
    let argv_validation_passed = validate_command_steps(stage_id, &command_steps).is_ok();
    let (
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
    ) = run_missing_input_probe(repo_root, stage_id, &required_inputs);

    let reason = format!(
        "row `{stage_id}` / `{}` renders concrete angsd argv with {} declared command step(s), {} required input(s), {} raw output id(s), and {} parser output id(s)",
        GOVERNED_ANGSD_TOOL_ID,
        command_steps.len(),
        required_inputs.len(),
        raw_output_ids.len(),
        parser_output_ids.len(),
    );

    Ok(VcfAngsdAdapterRow {
        tool_id: GOVERNED_ANGSD_TOOL_ID.to_string(),
        tool_status: GOVERNED_ANGSD_TOOL_STATUS.to_string(),
        stage_id: stage_id.to_string(),
        stage_status: catalog_row.support_status.clone(),
        benchmark_status: if benchmark_ready {
            "benchmark_ready".to_string()
        } else {
            "not_benchmark_ready".to_string()
        },
        adapter_id: adapter_id.to_string(),
        parser_id: parser_id.to_string(),
        corpus_id: GOVERNED_CORPUS_ID.to_string(),
        asset_profile_id: asset_profile_id_for_stage(stage).to_string(),
        command_contract_source,
        output_root,
        output_prefix,
        bam_member_paths,
        bam_list_path,
        reference_path,
        sites_path,
        likelihood_model,
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
    pub(super) reference_path: Option<String>,
    pub(super) sites_path: Option<String>,
    pub(super) likelihood_model: Option<String>,
    pub(super) output_prefix: String,
    pub(super) stage_output_ids: Vec<String>,
    pub(super) raw_output_ids: Vec<String>,
    pub(super) parser_output_ids: Vec<String>,
    pub(super) declared_outputs: Vec<VcfAngsdAdapterArtifact>,
    pub(super) command_steps: Vec<VcfAngsdAdapterCommandStep>,
}

pub(super) fn build_stage_adapter_contract(
    stage: VcfDomainStage,
    output_root: &str,
    inputs: &[VcfAngsdAdapterArtifact],
) -> Result<StageContract> {
    let contract = match stage {
        VcfDomainStage::CallGl => super::stages::call_gl::build(output_root, inputs)?,
        VcfDomainStage::CallPseudohaploid => {
            super::stages::call_pseudohaploid::build(output_root, inputs)?
        }
        other => {
            return Err(anyhow!(
                "VCF angsd adapter does not govern stage `{}` for Goal 236",
                other.as_str()
            ));
        }
    };

    Ok(contract)
}

pub(super) fn asset_profile_id_for_stage(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::CallGl | VcfDomainStage::CallPseudohaploid => "bam_bundle",
        other => panic!("unsupported angsd asset profile stage `{}`", other.as_str()),
    }
}

pub(super) fn uses_bam_inputs(stage: VcfDomainStage) -> bool {
    matches!(stage, VcfDomainStage::CallGl | VcfDomainStage::CallPseudohaploid)
}

pub(super) fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfAngsdAdapterArtifact {
    VcfAngsdAdapterArtifact {
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
) -> VcfAngsdAdapterCommandStep {
    VcfAngsdAdapterCommandStep {
        step_id: step_id.to_string(),
        step_kind: step_kind.to_string(),
        argv: argv.into_iter().map(str::to_string).collect(),
        declared_output_artifact_ids: declared_output_artifact_ids
            .iter()
            .map(|artifact_id| (*artifact_id).to_string())
            .collect(),
    }
}

pub(super) fn input_by_id<'a>(
    inputs: &'a [VcfAngsdAdapterArtifact],
    artifact_id: &str,
) -> Result<&'a str> {
    inputs
        .iter()
        .find(|input| input.artifact_id == artifact_id)
        .map(|input| input.path.as_str())
        .ok_or_else(|| anyhow!("VCF angsd adapter input `{artifact_id}` is missing"))
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
