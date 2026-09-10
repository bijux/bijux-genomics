//! Publish the source-to-metrics artifact relationship without rewriting native manifests.
use crate::imputation::model::LocalVcfImputeSmokeReport;
use crate::imputation_metrics::execution::source::SourceArtifacts;
use crate::imputation_metrics::model::ImputationMetricsSummary;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_REPORT_NAME;
use crate::imputation_metrics::model::LOCAL_VCF_IMPUTATION_METRICS_STAGE_ID;
use crate::publication::PublishedArtifactPaths;
use anyhow::Result;

pub(crate) fn publish_orchestration(
    source_report: &LocalVcfImputeSmokeReport,
    summary: &ImputationMetricsSummary,
    published: &PublishedArtifactPaths<'_>,
    artifacts: &SourceArtifacts,
) -> Result<()> {
    let imputation_metrics_path = published.execution.join(DEFAULT_OUTPUT_REPORT_NAME);
    let orchestration_manifest = serde_json::json!({
        "schema_version": "bijux.bench.local_vcf_imputation_metrics_orchestration.v1",
        "stage_id": LOCAL_VCF_IMPUTATION_METRICS_STAGE_ID,
        "tool_id": source_report.tool_id,
        "source_stage_id": source_report.stage_id,
        "status": summary.status,
        "input_fixture_id": source_report.input_fixture_id,
        "paths": {
            "imputed_vcf": crate::stage_result::path_relative_to_repo(published.repository, &artifacts.source_imputed_vcf),
            "imputed_tbi": crate::stage_result::path_relative_to_repo(published.repository, &artifacts.source_imputed_tbi),
            "imputation_qc_json": crate::stage_result::path_relative_to_repo(published.repository, &artifacts.source_imputation_qc),
            "imputation_accept_json": crate::stage_result::path_relative_to_repo(published.repository, &artifacts.source_imputation_accept),
            "imputation_manifest_json": crate::stage_result::path_relative_to_repo(published.repository, &artifacts.source_imputation_manifest),
            "logs_txt": crate::stage_result::path_relative_to_repo(published.repository, &artifacts.source_logs),
            "imputation_metrics_json": crate::stage_result::path_relative_to_repo(published.repository, &imputation_metrics_path)
        }
    });
    bijux_dna_infra::atomic_write_json(&artifacts.orchestration_manifest, &orchestration_manifest)?;

    Ok(())
}
