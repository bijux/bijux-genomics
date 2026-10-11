//! Owner: bijux-dna-bench-vcf
//! bcftools adapter commands contracts.
use super::{
    anyhow, governed_inputs_for_stage, run_missing_input_probe, validate_command_steps,
    validate_required_inputs, vcf_domain_stage_expected_output_ids, Path, PathBuf, Result,
    VcfAdapterInputs, VcfBcftoolsAdapterArtifact, VcfBcftoolsAdapterCommandStep,
    VcfBcftoolsAdapterRow, VcfDomainStage, VcfStageCatalogRow, VcfStageMatrixRow,
};

pub(super) fn build_bcftools_row(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    matrix_row: &VcfStageMatrixRow,
    catalog_row: &VcfStageCatalogRow,
) -> Result<VcfBcftoolsAdapterRow> {
    let stage = VcfDomainStage::try_from(matrix_row.stage_id.as_str())
        .map_err(|error| anyhow!("unknown VCF stage `{}`: {error}", matrix_row.stage_id))?;
    let output_root = format!(
        "artifacts/benchmark-smoke/readiness/adapters/{}/{}",
        matrix_row.tool_id, matrix_row.stage_id
    );
    let required_inputs = governed_inputs_for_stage(inputs, repo_root, stage, &output_root)?;
    validate_required_inputs(repo_root, &matrix_row.stage_id, &required_inputs)?;
    let stage_output_ids = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| {
            anyhow!("VCF stage `{}` is missing expected output ids", matrix_row.stage_id)
        })?
        .iter()
        .map(|output| (*output).to_string())
        .collect::<Vec<_>>();
    let StageContract { raw_output_ids, parser_output_ids, declared_outputs, command_steps } =
        build_stage_adapter_contract(stage, &matrix_row.stage_id, &output_root, &required_inputs)?;
    let argv_validation_passed =
        validate_command_steps(&matrix_row.stage_id, &command_steps).is_ok();
    let (
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
    ) = run_missing_input_probe(repo_root, &matrix_row.stage_id, &required_inputs);
    let benchmark_status = match catalog_row.support_status.as_str() {
        "supported" => "benchmark_ready",
        _ => "not_benchmark_ready",
    }
    .to_string();

    let reason = format!(
        "row `{}` / `{}` renders concrete bcftools argv with {} declared command step(s), {} required input(s), and {} stage output id(s)",
        matrix_row.stage_id,
        matrix_row.tool_id,
        command_steps.len(),
        required_inputs.len(),
        stage_output_ids.len()
    );

    Ok(VcfBcftoolsAdapterRow {
        tool_id: matrix_row.tool_id.clone(),
        stage_id: matrix_row.stage_id.clone(),
        support_status: catalog_row.support_status.clone(),
        benchmark_status,
        adapter_id: matrix_row.adapter_id.clone(),
        parser_id: matrix_row.parser_id.clone(),
        corpus_id: matrix_row.corpus_id.clone(),
        asset_profile_id: matrix_row.asset_profile_id.clone(),
        output_root,
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
    pub(super) raw_output_ids: Vec<String>,
    pub(super) parser_output_ids: Vec<String>,
    pub(super) declared_outputs: Vec<VcfBcftoolsAdapterArtifact>,
    pub(super) command_steps: Vec<VcfBcftoolsAdapterCommandStep>,
}

pub(super) fn build_stage_adapter_contract(
    stage: VcfDomainStage,
    stage_id: &str,
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    let contract = match stage {
        VcfDomainStage::Qc => super::stages::qc::build(output_root, inputs)?,
        VcfDomainStage::PrepareReferencePanel => {
            super::stages::prepare_reference_panel::build(output_root, inputs)?
        }
        VcfDomainStage::Call => super::stages::call::build(output_root, inputs)?,
        VcfDomainStage::CallDiploid => super::stages::call_diploid::build(output_root, inputs)?,
        VcfDomainStage::CallGl => super::stages::call_gl::build(output_root, inputs)?,
        VcfDomainStage::CallPseudohaploid => {
            super::stages::call_pseudohaploid::build(output_root, inputs)?
        }
        VcfDomainStage::DamageFilter => super::stages::damage_filter::build(output_root, inputs)?,
        VcfDomainStage::Filter => super::stages::filter::build(output_root, inputs)?,
        VcfDomainStage::GlPropagation => super::stages::gl_propagation::build(output_root, inputs)?,
        VcfDomainStage::Postprocess => super::stages::postprocess::build(output_root, inputs)?,
        VcfDomainStage::Stats => super::stages::stats::build(output_root, inputs)?,
        other => {
            return Err(anyhow!(
                "VCF bcftools adapter does not govern stage `{}` for Goal 235",
                other.as_str()
            ));
        }
    };

    if contract.command_steps.is_empty() {
        return Err(anyhow!(
            "VCF bcftools adapter row `{stage_id}` must keep at least one executable command step"
        ));
    }
    Ok(contract)
}

pub(super) fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfBcftoolsAdapterArtifact {
    VcfBcftoolsAdapterArtifact {
        artifact_id: artifact_id.to_string(),
        role: role.to_string(),
        path: path.to_string(),
    }
}

pub(super) fn step(
    step_id: &str,
    step_kind: &str,
    argv: Vec<&str>,
    consumes_previous_stdout: bool,
    declared_output_artifact_ids: &[&str],
) -> VcfBcftoolsAdapterCommandStep {
    VcfBcftoolsAdapterCommandStep {
        step_id: step_id.to_string(),
        step_kind: step_kind.to_string(),
        argv: argv.into_iter().map(str::to_string).collect(),
        consumes_previous_stdout,
        declared_output_artifact_ids: declared_output_artifact_ids
            .iter()
            .map(|artifact_id| (*artifact_id).to_string())
            .collect(),
    }
}

pub(super) fn step_strings(
    step_id: &str,
    step_kind: &str,
    argv: Vec<String>,
    consumes_previous_stdout: bool,
    declared_output_artifact_ids: &[&str],
) -> VcfBcftoolsAdapterCommandStep {
    VcfBcftoolsAdapterCommandStep {
        step_id: step_id.to_string(),
        step_kind: step_kind.to_string(),
        argv,
        consumes_previous_stdout,
        declared_output_artifact_ids: declared_output_artifact_ids
            .iter()
            .map(|artifact_id| (*artifact_id).to_string())
            .collect(),
    }
}

pub(super) fn input_by_id<'a>(
    inputs: &'a [VcfBcftoolsAdapterArtifact],
    artifact_id: &str,
) -> Result<&'a str> {
    inputs
        .iter()
        .find(|input| input.artifact_id == artifact_id)
        .map(|input| input.path.as_str())
        .ok_or_else(|| anyhow!("VCF bcftools adapter input `{artifact_id}` is missing"))
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
