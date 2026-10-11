//! Project validated evidence into the existing public report schema.
use crate::publication::ReportContext;
use crate::qc::execution::artifacts::StageArtifacts;
use crate::qc::execution::inputs::PreparedInputs;
use crate::qc::model::GovernedVcfQcSmokeContract;
use crate::qc::model::LocalVcfQcSmokeReport;
use crate::qc::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::qc::model::DEFAULT_OUTPUT_QC_NAME;
use crate::qc::model::DEFAULT_STAGE_RESULT_NAME;
use crate::qc::model::LOCAL_VCF_QC_SMOKE_COMMAND;
use crate::qc::model::LOCAL_VCF_QC_SMOKE_SCHEMA_VERSION;
use crate::qc::observations::stage::StageEvidence;
use anyhow::Result;

pub(crate) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfQcSmokeContract,
    prepared: &PreparedInputs,
    artifacts: &StageArtifacts,
    evidence: &StageEvidence,
) -> Result<LocalVcfQcSmokeReport> {
    let metrics = &evidence.metrics;
    let output_root = context.paths.execution;
    let published = &context.paths;
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
    let report = LocalVcfQcSmokeReport {
        schema_version: LOCAL_VCF_QC_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_QC_SMOKE_COMMAND} --tool-id {}", contract.tool_id),
        stage_id: contract.stage_id.clone(),
        tool_id: contract.tool_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        sample_name: contract.sample_name.clone(),
        input_vcf_path: published.path(input_vcf)?,
        output_root: published.path(output_root)?,
        qc_json_path: published.path(&qc_json_path)?,
        qc_summary_path: published.path(qc_summary_path)?,
        qc_tables_path: published.path(qc_tables_path)?,
        imputation_qc_path: published.path(imputation_qc_path)?,
        warnings_path: published.path(warnings_path)?,
        qc_histograms_path: published.path(qc_histograms_path)?,
        metrics_path: published.path(&metrics_path)?,
        stage_result_manifest_path: published.path(&stage_result_manifest_path)?,
        started_at: started_at.to_owned(),
        finished_at: finished_at.to_owned(),
        elapsed_seconds,
        exit_code: metrics.exit_code,
        sample_missingness: metrics.sample_missingness.clone(),
        variant_missingness: metrics.variant_missingness.clone(),
        maf_summary: metrics.maf_summary.clone(),
        heterozygosity: metrics.heterozygosity.clone(),
        hwe_summary: metrics.hwe_summary.clone(),
        excluded_samples: metrics.excluded_samples.clone(),
        excluded_variants: metrics.excluded_variants.clone(),
        sample_missingness_exclusion_threshold: metrics.sample_missingness_exclusion_threshold,
        variant_missingness_exclusion_threshold: metrics.variant_missingness_exclusion_threshold,
    };
    Ok(report)
}
