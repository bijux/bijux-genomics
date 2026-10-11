//! Publish declared execution outputs using actual execution addresses.
use crate::publication::ReportContext;
use crate::qc::execution::artifacts::StageArtifacts;
use crate::qc::execution::inputs::PreparedInputs;
use crate::qc::model::GovernedVcfQcSmokeContract;
use crate::qc::model::LocalVcfQcSummary;
use crate::qc::model::DEFAULT_OUTPUT_HISTOGRAMS_NAME;
use crate::qc::model::DEFAULT_OUTPUT_IMPUTATION_QC_NAME;
use crate::qc::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::qc::model::DEFAULT_OUTPUT_QC_NAME;
use crate::qc::model::DEFAULT_OUTPUT_QC_TABLES_NAME;
use crate::qc::model::DEFAULT_OUTPUT_SUMMARY_NAME;
use crate::qc::model::DEFAULT_OUTPUT_WARNINGS_NAME;
use crate::qc::model::DEFAULT_STAGE_RESULT_NAME;
use crate::qc::model::LOCAL_VCF_QC_SMOKE_COMMAND;
use crate::qc::publication::projections::materialize_governed_qc_contract_outputs;
use crate::qc::publication::stage_result::build_stage_result_manifest;
use crate::stage_result::validate_stage_result_manifest;
use anyhow::Result;

pub(crate) fn publish_stage_result(
    context: &ReportContext<'_>,
    contract: &GovernedVcfQcSmokeContract,
    prepared: &PreparedInputs,
    artifacts: &StageArtifacts,
    summary: &LocalVcfQcSummary,
) -> Result<()> {
    let repo_root = context.paths.repository;
    let output_root = context.paths.execution;
    let input_vcf = &prepared.input_vcf;
    let metrics_path = context.paths.execution.join(DEFAULT_OUTPUT_METRICS_NAME);
    let stage_result_manifest_path = context.paths.execution.join(DEFAULT_STAGE_RESULT_NAME);
    let qc_json_path = context.paths.execution.join(DEFAULT_OUTPUT_QC_NAME);
    let started_at = context.started_at;
    let finished_at = context.finished_at;
    let elapsed_seconds = context.elapsed_seconds;
    let qc_summary_path = &artifacts.qc_summary;
    let qc_tables_path = &artifacts.qc_tables;
    let imputation_qc_path = &artifacts.imputation_qc;
    let warnings_path = &artifacts.warnings;
    let qc_histograms_path = &artifacts.qc_histograms;
    let governed_contract_outputs = materialize_governed_qc_contract_outputs(
        output_root,
        input_vcf,
        summary,
        contract.report_format,
    )?;
    super::provenance::publish_provenance(output_root, contract.report_format)?;
    let mut declared_outputs = vec![
        (
            "qc_report_json",
            DEFAULT_OUTPUT_QC_NAME.to_string(),
            qc_json_path.as_path(),
            "report_output",
        ),
        (
            "qc_summary_json",
            DEFAULT_OUTPUT_SUMMARY_NAME.to_string(),
            qc_summary_path.as_path(),
            "report_output",
        ),
        (
            "qc_tables_tsv",
            DEFAULT_OUTPUT_QC_TABLES_NAME.to_string(),
            qc_tables_path.as_path(),
            "report_output",
        ),
        (
            "imputation_qc_tsv",
            DEFAULT_OUTPUT_IMPUTATION_QC_NAME.to_string(),
            imputation_qc_path.as_path(),
            "report_output",
        ),
        (
            "warnings_json",
            DEFAULT_OUTPUT_WARNINGS_NAME.to_string(),
            warnings_path.as_path(),
            "report_output",
        ),
        (
            "qc_histograms_json",
            DEFAULT_OUTPUT_HISTOGRAMS_NAME.to_string(),
            qc_histograms_path.as_path(),
            "report_output",
        ),
        (
            "metrics_json",
            DEFAULT_OUTPUT_METRICS_NAME.to_string(),
            metrics_path.as_path(),
            "report_output",
        ),
        ("qc_report", DEFAULT_OUTPUT_QC_NAME.to_string(), qc_json_path.as_path(), "report_output"),
    ];
    for output in &governed_contract_outputs {
        declared_outputs.push((
            output.artifact_id,
            output.declared_path.clone(),
            output.path.as_path(),
            output.role,
        ));
    }
    let stage_result_manifest = build_stage_result_manifest(
        repo_root,
        contract,
        &format!("{LOCAL_VCF_QC_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        &declared_outputs,
        started_at,
        finished_at,
        elapsed_seconds,
    );
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;
    Ok(())
}
