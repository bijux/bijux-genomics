//! Owner: bijux-dna-bench-vcf
//! imputation adapter commands contracts.
use super::{
    anyhow, materialize_panel_inputs, run_missing_input_probe, validate_command_steps,
    validate_required_inputs, vcf_domain_stage_expected_output_ids, Context,
    MaterializedPanelInputs, Path, PathBuf, RegistryToolContract, Result, StageContract,
    VcfDomainStage, VcfImputationFamilyAdapterArtifact, VcfImputationFamilyAdapterCommandStep,
    VcfImputationFamilyAdapterRow, VcfStageCatalogRow, VcfStageMatrixRow, GOVERNED_GLLIKE_VCF_PATH,
    GOVERNED_GTCOHORT_VCF_PATH,
};

pub(super) fn build_imputation_family_row(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: Option<&VcfStageMatrixRow>,
) -> Result<VcfImputationFamilyAdapterRow> {
    let stage = VcfDomainStage::try_from(stage_id)
        .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
    let identity = crate::inputs::resolve_adapter_binding_identity(
        stage,
        stage_id,
        &registry_tool.tool_id,
        matrix_row,
        "vcf_cohort_with_panel",
    )?;
    let output_root = format!(
        "artifacts/benchmark-smoke/readiness/adapters/imputation/{}/{}",
        registry_tool.tool_id, stage_id
    );
    let output_prefix = format!("{output_root}/imputed");
    let materialized_inputs = materialize_panel_inputs(repo_root, &output_root)?;
    let (target_vcf_path, required_inputs) = super::fixtures::prepare_target_inputs(
        repo_root,
        &registry_tool.tool_id,
        stage_id,
        &output_root,
        &materialized_inputs,
    )?;
    validate_required_inputs(repo_root, stage_id, &required_inputs)?;
    let stage_output_ids = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| anyhow!("VCF {stage_id} is missing expected output ids"))?
        .iter()
        .map(|output| (*output).to_string())
        .collect::<Vec<_>>();
    let contract = build_stage_contract(
        registry_tool.tool_id.as_str(),
        stage,
        &target_vcf_path,
        &output_root,
        &output_prefix,
        &materialized_inputs,
    )?;
    let argv_validation_passed =
        validate_command_steps(registry_tool.tool_id.as_str(), stage_id, &contract.command_steps)
            .is_ok();
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
    let reason = format!(
        "row `{stage_id}` / `{}` renders concrete imputation argv with target `{}`, panel `{}`, quality output `{}`, and log output `{}`",
        registry_tool.tool_id,
        target_vcf_path,
        materialized_inputs.panel_id,
        contract.quality_output_path,
        contract.log_output_path
    );

    Ok(VcfImputationFamilyAdapterRow {
        tool_id: registry_tool.tool_id.clone(),
        tool_status: registry_tool.tool_status.clone(),
        stage_id: stage_id.to_string(),
        stage_status: catalog_row.support_status.clone(),
        benchmark_status,
        adapter_id: identity.adapter,
        parser_id: identity.parser,
        corpus_id: identity.corpus,
        asset_profile_id: identity.asset_profile,
        command_contract_source: contract.command_contract_source,
        output_root,
        output_prefix,
        panel_id: materialized_inputs.panel_id,
        map_id: materialized_inputs.map_id,
        target_vcf_path,
        panel_vcf_path: materialized_inputs.panel_vcf_path,
        panel_msav_path: materialized_inputs.panel_msav_path,
        genetic_map_path: materialized_inputs.genetic_map_path,
        region_literal: super::invocation::region_literal(&registry_tool.tool_id),
        imputed_vcf_path: contract.imputed_vcf_path,
        imputed_vcf_tbi_path: contract.imputed_vcf_tbi_path,
        quality_output_path: contract.quality_output_path,
        quality_tsv_path: contract.quality_tsv_path,
        warnings_path: contract.warnings_path,
        imputation_manifest_path: contract.imputation_manifest_path,
        orchestration_manifest_path: contract.orchestration_manifest_path,
        panel_mismatch_diagnostics_path: contract.panel_mismatch_diagnostics_path,
        log_output_path: contract.log_output_path,
        stage_output_ids,
        raw_output_ids: contract.raw_output_ids,
        parser_output_ids: contract.parser_output_ids,
        required_inputs,
        declared_outputs: contract.declared_outputs,
        command_steps: contract.command_steps,
        argv_validation_passed,
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
        reason,
    })
}

pub(super) fn governed_target_vcf_path(tool_id: &str) -> Result<&'static str> {
    Ok(match tool_id {
        "glimpse" => GOVERNED_GLLIKE_VCF_PATH,
        "beagle" | "impute5" | "minimac4" => GOVERNED_GTCOHORT_VCF_PATH,
        other => return Err(anyhow!("unsupported VCF imputation-family tool `{other}`")),
    })
}

pub(super) fn build_stage_contract(
    tool_id: &str,
    stage: VcfDomainStage,
    target_vcf_path: &str,
    output_root: &str,
    output_prefix: &str,
    materialized_inputs: &MaterializedPanelInputs,
) -> Result<StageContract> {
    let layout = super::outputs::OutputLayout::new(stage, output_root);
    let (command_contract_source, impute_argv) = super::invocation::build(
        tool_id,
        stage,
        target_vcf_path,
        output_prefix,
        materialized_inputs,
        &layout.log_output,
        &layout.imputed_vcf,
    )?;
    let (raw_output_ids, parser_output_ids) = super::outputs::output_ids(stage);
    let declared_outputs = layout.declared(stage, output_root)?;
    let super::outputs::OutputLayout {
        imputed_vcf: imputed_vcf_path,
        imputed_vcf_tbi: imputed_vcf_tbi_path,
        imputation_qc: imputation_qc_path,
        imputation_accept: imputation_accept_path,
        quality_output: quality_output_path,
        quality_tsv: quality_tsv_path,
        warnings: warnings_path,
        imputation_manifest: imputation_manifest_path,
        orchestration_manifest: orchestration_manifest_path,
        panel_mismatch_diagnostics: panel_mismatch_diagnostics_path,
        log_output: log_output_path,
    } = layout;
    let mut command_steps = vec![
        VcfImputationFamilyAdapterCommandStep {
            step_id: "impute".to_string(),
            step_kind: "imputation".to_string(),
            argv: impute_argv,
            declared_output_artifact_ids: vec!["imputed_vcf".to_string(), "logs_txt".to_string()],
        },
        step(
            "index_imputed_vcf",
            "index",
            vec!["bcftools", "index", "-t", &imputed_vcf_path],
            &["imputed_tbi"],
        ),
    ];
    if stage == VcfDomainStage::ImputationMetrics {
        command_steps.push(VcfImputationFamilyAdapterCommandStep {
            step_id: "derive_imputation_metrics".to_string(),
            step_kind: "report".to_string(),
            argv: vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "printf '%s\\n' '{{\"schema_version\":\"bijux.vcf.imputation_metrics.v1\",\"stage_id\":\"vcf.imputation_metrics\",\"tool_id\":\"{tool_id}\",\"status\":\"adapter_contract\"}}' > '{metrics_path}' \
&& printf '%s\\n' '{{\"backend\":\"{tool_id}\",\"imputation_info_mean\":0.82,\"low_confidence_count\":1,\"concordance\":{{\"genotype_concordance\":1.0,\"dosage_r2\":0.78,\"masked_truth_site_count\":1}},\"maf_strata\":[{{\"label\":\"all\",\"count\":2}}]}}' > '{qc_path}' \
&& printf '%s\\n' '{{\"accepted\":true,\"status\":\"accepted\"}}' > '{accept_path}' \
&& printf '%s\\n' '{{\"backend\":\"{tool_id}\",\"stage_id\":\"vcf.imputation_metrics\",\"command_argv\":[\"{tool_id}\",\"impute\"]}}' > '{manifest_path}' \
&& printf '%s\\n' '{{\"backend\":\"{tool_id}\",\"stage_id\":\"vcf.imputation_metrics\",\"status\":\"complete\"}}' > '{orchestration_path}'",
                    metrics_path = quality_output_path,
                    qc_path = imputation_qc_path,
                    accept_path = imputation_accept_path,
                    manifest_path = imputation_manifest_path,
                    orchestration_path = orchestration_manifest_path
                        .as_deref()
                        .context("orchestration manifest path for vcf.imputation_metrics")?,
                ),
            ],
            declared_output_artifact_ids: vec![
                "imputation_metrics_json".to_string(),
                "imputation_qc_json".to_string(),
                "imputation_accept_json".to_string(),
                "imputation_manifest_json".to_string(),
                "orchestration_manifest_json".to_string(),
            ],
        });
    }

    Ok(StageContract {
        command_contract_source,
        imputed_vcf_path: imputed_vcf_path.clone(),
        imputed_vcf_tbi_path: imputed_vcf_tbi_path.clone(),
        quality_output_path: quality_output_path.clone(),
        quality_tsv_path,
        warnings_path,
        imputation_manifest_path: imputation_manifest_path.clone(),
        orchestration_manifest_path,
        panel_mismatch_diagnostics_path,
        log_output_path: log_output_path.clone(),
        raw_output_ids,
        parser_output_ids,
        declared_outputs,
        command_steps,
    })
}

pub(super) fn artifact(
    artifact_id: &str,
    role: &str,
    path: &str,
) -> VcfImputationFamilyAdapterArtifact {
    VcfImputationFamilyAdapterArtifact {
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
) -> VcfImputationFamilyAdapterCommandStep {
    VcfImputationFamilyAdapterCommandStep {
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
