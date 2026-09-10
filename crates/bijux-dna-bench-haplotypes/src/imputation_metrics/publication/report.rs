//! Project normalized quality evidence into the persisted report.
use crate::imputation::model::LocalVcfImputeSmokeReport;
use crate::imputation_metrics::execution::source::SourceArtifacts;
use crate::imputation_metrics::model::ImputationMetricsSummary;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_REPORT_NAME;
use crate::imputation_metrics::model::DEFAULT_STAGE_RESULT_NAME;
use crate::imputation_metrics::model::LOCAL_VCF_IMPUTATION_METRICS_COMMAND;
use crate::imputation_metrics::model::LOCAL_VCF_IMPUTATION_METRICS_STAGE_ID;
use crate::publication::PublishedArtifactPaths;
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalVcfImputationMetricsSmokeReport as LocalVcfImputationMetricsSmokeReport;
use bijux_dna_bench_model::BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1;

pub(crate) fn build_report_payload(
    source_report: &LocalVcfImputeSmokeReport,
    summary: &ImputationMetricsSummary,
    published: &PublishedArtifactPaths<'_>,
    artifacts: &SourceArtifacts,
    started_at: &str,
) -> Result<LocalVcfImputationMetricsSmokeReport> {
    let output_root = published.execution;
    let imputation_metrics_path = output_root.join(DEFAULT_OUTPUT_REPORT_NAME);
    let report_payload = LocalVcfImputationMetricsSmokeReport {
        schema_version: BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1.to_string(),
        command: format!(
            "{LOCAL_VCF_IMPUTATION_METRICS_COMMAND} --tool-id {}",
            source_report.tool_id
        ),
        stage_id: LOCAL_VCF_IMPUTATION_METRICS_STAGE_ID.to_string(),
        tool_id: source_report.tool_id.clone(),
        corpus_id: source_report.corpus_id.clone(),
        input_fixture_id: source_report.input_fixture_id.clone(),
        panel_id: source_report.panel_id.clone(),
        map_id: source_report.map_id.clone(),
        output_root: published.path(output_root)?,
        imputation_metrics_path: published.path(&imputation_metrics_path)?,
        source_imputation_qc_path: published.path(&artifacts.source_imputation_qc)?,
        source_impute_smoke_metrics_path: published.path(&artifacts.source_impute_smoke_metrics)?,
        source_imputation_manifest_path: published.path(&artifacts.source_imputation_manifest)?,
        stage_result_manifest_path: published.path(&output_root.join(DEFAULT_STAGE_RESULT_NAME))?,
        started_at: started_at.to_owned(),
        finished_at: String::new(),
        elapsed_seconds: 0.0,
        exit_code: 0,
        concordance: summary.concordance,
        mean_info_score: summary.mean_info_score,
        r2_available: summary.r2_available,
        dosage_r2: summary.dosage_r2,
        low_confidence_sites: summary.low_confidence_sites,
        masked_truth_sites: summary.masked_truth_sites,
        quality_field_availability: summary.quality_field_availability.clone(),
        missing_quality_fields: summary.missing_quality_fields.clone(),
        status: summary.status.clone(),
    };
    Ok(report_payload)
}
