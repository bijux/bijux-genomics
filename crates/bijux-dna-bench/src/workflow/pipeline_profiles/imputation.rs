use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use super::{
    require_list_contains, require_profile_stage, require_vcf_matrix_binding,
    LocalPipelineDagConfig, LocalPipelineDagValidationProfileReport, PipelineProfileEvidence,
};

pub(super) fn validate_reference_panel_imputation_profile(
    evidence: &dyn PipelineProfileEvidence,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "reference_panel_imputation";

    if config.default_corpus_id != "vcf_production_regression" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed VCF corpus `vcf_production_regression`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let prepare_reference_panel =
        require_profile_stage(&stage_index, PROFILE_ID, "vcf.prepare_reference_panel")?;
    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;
    let vcf_phasing = require_profile_stage(&stage_index, PROFILE_ID, "vcf.phasing")?;
    let vcf_impute = require_profile_stage(&stage_index, PROFILE_ID, "vcf.impute")?;
    let vcf_imputation_metrics =
        require_profile_stage(&stage_index, PROFILE_ID, "vcf.imputation_metrics")?;

    let matrix_index = evidence.vcf_stage_matrix()?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.prepare_reference_panel",
        "vcf_reference_panel",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.qc",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.phasing",
        "vcf_cohort_with_panel",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.impute",
        "vcf_cohort_with_panel",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.imputation_metrics",
        "vcf_cohort_with_panel",
        "vcf_production_regression",
    )?;

    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "reference_panel_id_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed panel-id contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "reference_panel_lock_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed reference-panel lock contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "genetic_map_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed genetic map contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "reference_fasta_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed reference FASTA contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "reference_fai_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed reference FASTA index contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "reference_dict_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed reference dictionary contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.outputs,
        "prepared_panel_vcf",
        PROFILE_ID,
        "vcf.prepare_reference_panel must produce the prepared panel VCF handoff",
    )?;
    require_list_contains(
        &prepare_reference_panel.outputs,
        "prepared_panel_vcf_tbi",
        PROFILE_ID,
        "vcf.prepare_reference_panel must produce the prepared panel tabix handoff",
    )?;
    require_list_contains(
        &prepare_reference_panel.outputs,
        "prepared_panel_panel_id",
        PROFILE_ID,
        "vcf.prepare_reference_panel must surface explicit panel identity for downstream phasing and imputation",
    )?;

    require_list_contains(
        &vcf_qc.outputs,
        "qc_target_vcf",
        PROFILE_ID,
        "vcf.qc must keep the target-cohort VCF handoff explicit for optional prephasing",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_target_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must keep the target-cohort tabix handoff explicit for optional prephasing",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "phasing_requirement_decision_json",
        PROFILE_ID,
        "vcf.qc must emit the governed phasing-requirement decision handoff",
    )?;

    require_list_contains(
        &vcf_phasing.depends_on,
        "vcf.prepare_reference_panel",
        PROFILE_ID,
        "vcf.phasing must remain downstream of prepared-panel materialization",
    )?;
    require_list_contains(
        &vcf_phasing.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.phasing must remain downstream of target-cohort QC",
    )?;
    require_list_contains(
        &vcf_phasing.external_inputs,
        "reference_panel_id_contract",
        PROFILE_ID,
        "vcf.phasing must declare the governed panel-id contract",
    )?;
    require_list_contains(
        &vcf_phasing.external_inputs,
        "reference_panel_lock_contract",
        PROFILE_ID,
        "vcf.phasing must declare the governed reference-panel lock contract",
    )?;
    require_list_contains(
        &vcf_phasing.external_inputs,
        "genetic_map_contract",
        PROFILE_ID,
        "vcf.phasing must declare the governed genetic map contract",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "qc_target_vcf",
        PROFILE_ID,
        "vcf.phasing must consume the QC-qualified target VCF handoff",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "phasing_requirement_decision_json",
        PROFILE_ID,
        "vcf.phasing must consume the explicit phasing decision handoff",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "prepared_panel_vcf",
        PROFILE_ID,
        "vcf.phasing must consume the prepared reference panel VCF handoff",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "prepared_panel_panel_id",
        PROFILE_ID,
        "vcf.phasing must consume explicit prepared-panel identity",
    )?;

    require_list_contains(
        &vcf_impute.depends_on,
        "vcf.prepare_reference_panel",
        PROFILE_ID,
        "vcf.impute must remain downstream of prepared-panel materialization",
    )?;
    require_list_contains(
        &vcf_impute.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.impute must remain downstream of target-cohort QC",
    )?;
    require_list_contains(
        &vcf_impute.depends_on,
        "vcf.phasing",
        PROFILE_ID,
        "vcf.impute must remain downstream of the optional prephasing branch",
    )?;
    require_list_contains(
        &vcf_impute.external_inputs,
        "reference_panel_id_contract",
        PROFILE_ID,
        "vcf.impute must not run without the governed panel-id contract",
    )?;
    require_list_contains(
        &vcf_impute.external_inputs,
        "reference_panel_lock_contract",
        PROFILE_ID,
        "vcf.impute must declare the governed reference-panel lock contract",
    )?;
    require_list_contains(
        &vcf_impute.external_inputs,
        "genetic_map_contract",
        PROFILE_ID,
        "vcf.impute must declare the governed genetic map contract",
    )?;
    require_list_contains(
        &vcf_impute.external_inputs,
        "reference_fasta_contract",
        PROFILE_ID,
        "vcf.impute must declare the governed imputation reference FASTA contract",
    )?;
    require_list_contains(
        &vcf_impute.external_inputs,
        "reference_fai_contract",
        PROFILE_ID,
        "vcf.impute must declare the governed imputation reference FASTA index contract",
    )?;
    require_list_contains(
        &vcf_impute.upstream_inputs,
        "qc_target_vcf",
        PROFILE_ID,
        "vcf.impute must consume the QC-qualified target VCF fallback handoff",
    )?;
    require_list_contains(
        &vcf_impute.upstream_inputs,
        "phased_vcf",
        PROFILE_ID,
        "vcf.impute must consume the phased VCF run-path handoff",
    )?;
    require_list_contains(
        &vcf_impute.upstream_inputs,
        "phasing_requirement_decision_json",
        PROFILE_ID,
        "vcf.impute must consume the explicit phasing-decision handoff",
    )?;
    require_list_contains(
        &vcf_impute.upstream_inputs,
        "prepared_panel_vcf",
        PROFILE_ID,
        "vcf.impute must consume the prepared panel VCF handoff",
    )?;
    require_list_contains(
        &vcf_impute.upstream_inputs,
        "prepared_panel_panel_id",
        PROFILE_ID,
        "vcf.impute must consume explicit panel identity from prepared-panel outputs",
    )?;

    require_list_contains(
        &vcf_imputation_metrics.depends_on,
        "vcf.prepare_reference_panel",
        PROFILE_ID,
        "vcf.imputation_metrics must remain downstream of prepared-panel provenance",
    )?;
    require_list_contains(
        &vcf_imputation_metrics.depends_on,
        "vcf.impute",
        PROFILE_ID,
        "vcf.imputation_metrics must remain downstream of the imputation execution stage",
    )?;
    require_list_contains(
        &vcf_imputation_metrics.external_inputs,
        "reference_panel_id_contract",
        PROFILE_ID,
        "vcf.imputation_metrics must declare the governed panel-id contract",
    )?;
    require_list_contains(
        &vcf_imputation_metrics.external_inputs,
        "reference_panel_lock_contract",
        PROFILE_ID,
        "vcf.imputation_metrics must declare the governed reference-panel lock contract",
    )?;
    require_list_contains(
        &vcf_imputation_metrics.external_inputs,
        "genetic_map_contract",
        PROFILE_ID,
        "vcf.imputation_metrics must declare the governed genetic map contract",
    )?;
    require_list_contains(
        &vcf_imputation_metrics.upstream_inputs,
        "prepared_panel_panel_id",
        PROFILE_ID,
        "vcf.imputation_metrics must retain explicit panel identity in the downstream metrics branch",
    )?;
    require_list_contains(
        &vcf_imputation_metrics.upstream_inputs,
        "imputation_manifest_json",
        PROFILE_ID,
        "vcf.imputation_metrics must consume explicit imputation manifest evidence",
    )?;
    require_list_contains(
        &vcf_imputation_metrics.upstream_inputs,
        "imputation_qc_json",
        PROFILE_ID,
        "vcf.imputation_metrics must consume explicit imputation QC evidence",
    )?;

    let checks = vec![
        "default corpus anchored to vcf_production_regression for governed target-cohort VCF inputs"
            .to_string(),
        "vcf.prepare_reference_panel stays bound to the vcf_reference_panel asset profile"
            .to_string(),
        "vcf.qc stays bound to the vcf_cohort asset profile".to_string(),
        "vcf.phasing, vcf.impute, and vcf.imputation_metrics stay bound to the vcf_cohort_with_panel asset profile"
            .to_string(),
        "vcf.prepare_reference_panel keeps panel, map, and reference contracts explicit"
            .to_string(),
        "vcf.phasing consumes the qc-qualified target VCF plus prepared-panel identity for the optional prephasing branch"
            .to_string(),
        "vcf.impute consumes both the qc-target fallback path and phased run path with explicit panel identity"
            .to_string(),
        "vcf.imputation_metrics stays downstream of imputation execution with panel-provenance and imputation-qc evidence"
            .to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}
