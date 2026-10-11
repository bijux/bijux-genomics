//! Publish the declared imputation quality artifact contract.
use crate::imputation::model::LocalVcfImputeSmokeReport;
use crate::imputation_metrics::execution::source::SourceArtifacts;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_ORCHESTRATION_MANIFEST_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_REPORT_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_ACCEPT_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_QC_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_SMOKE_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_TBI_NAME;
use crate::imputation_metrics::model::DEFAULT_OUTPUT_SOURCE_VCF_NAME;
use crate::imputation_metrics::model::DEFAULT_STAGE_RESULT_NAME;
use crate::imputation_metrics::model::LOCAL_VCF_IMPUTATION_METRICS_COMMAND;
use crate::imputation_metrics::model::LOCAL_VCF_IMPUTATION_METRICS_STAGE_ID;
use crate::publication::ReportContext;
use crate::stage_result::validate_stage_result_manifest;
use crate::stage_result::BenchStageResultCommandV1;
use crate::stage_result::BenchStageResultManifestV1;
use crate::stage_result::BenchStageResultOutputV1;
use crate::stage_result::BenchStageResultResourceMetricSource;
use crate::stage_result::BenchStageResultResourceMetricsV1;
use crate::stage_result::BenchStageResultRuntimeV1;
use crate::stage_result::BenchStageResultStatus;
use crate::stage_result::BenchStageResultToolV1;
use crate::stage_result::BENCH_STAGE_RESULT_SCHEMA_VERSION;
use anyhow::Result;

pub(crate) fn publish_stage_result(
    context: &ReportContext<'_>,
    source_report: &LocalVcfImputeSmokeReport,
    artifacts: &SourceArtifacts,
) -> Result<()> {
    let stage_result_manifest_path = context.paths.execution.join(DEFAULT_STAGE_RESULT_NAME);
    let stage_result_manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: LOCAL_VCF_IMPUTATION_METRICS_STAGE_ID.to_string(),
        tool: BenchStageResultToolV1 { id: source_report.tool_id.clone() },
        command: BenchStageResultCommandV1 {
            rendered: format!(
                "{LOCAL_VCF_IMPUTATION_METRICS_COMMAND} --tool-id {}",
                source_report.tool_id
            ),
        },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: context.started_at.to_owned(),
            finished_at: context.finished_at.to_owned(),
            elapsed_seconds: context.elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: output_contract(context, artifacts),
    };
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;
    Ok(())
}

fn output_contract(
    context: &ReportContext<'_>,
    artifacts: &SourceArtifacts,
) -> Vec<BenchStageResultOutputV1> {
    let imputation_metrics_path = context.paths.execution.join(DEFAULT_OUTPUT_REPORT_NAME);
    [
        (
            "imputed_vcf".to_string(),
            DEFAULT_OUTPUT_SOURCE_VCF_NAME.to_string(),
            crate::stage_result::path_relative_to_repo(
                context.paths.repository,
                &artifacts.source_imputed_vcf,
            ),
            "vcf_output".to_string(),
        ),
        (
            "imputed_tbi".to_string(),
            DEFAULT_OUTPUT_SOURCE_TBI_NAME.to_string(),
            crate::stage_result::path_relative_to_repo(
                context.paths.repository,
                &artifacts.source_imputed_tbi,
            ),
            "index_output".to_string(),
        ),
        (
            "imputation_metrics_json".to_string(),
            DEFAULT_OUTPUT_REPORT_NAME.to_string(),
            crate::stage_result::path_relative_to_repo(
                context.paths.repository,
                &imputation_metrics_path,
            ),
            "report_output".to_string(),
        ),
        (
            "imputation_qc_json".to_string(),
            DEFAULT_OUTPUT_SOURCE_QC_NAME.to_string(),
            crate::stage_result::path_relative_to_repo(
                context.paths.repository,
                &artifacts.source_imputation_qc,
            ),
            "report_output".to_string(),
        ),
        (
            "imputation_accept_json".to_string(),
            DEFAULT_OUTPUT_SOURCE_ACCEPT_NAME.to_string(),
            crate::stage_result::path_relative_to_repo(
                context.paths.repository,
                &artifacts.source_imputation_accept,
            ),
            "report_output".to_string(),
        ),
        (
            "imputation_manifest_json".to_string(),
            DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME.to_string(),
            crate::stage_result::path_relative_to_repo(
                context.paths.repository,
                &artifacts.source_imputation_manifest,
            ),
            "report_output".to_string(),
        ),
        (
            "logs_txt".to_string(),
            DEFAULT_OUTPUT_SOURCE_LOGS_NAME.to_string(),
            crate::stage_result::path_relative_to_repo(
                context.paths.repository,
                &artifacts.source_logs,
            ),
            "log_output".to_string(),
        ),
        (
            "orchestration_manifest_json".to_string(),
            DEFAULT_OUTPUT_ORCHESTRATION_MANIFEST_NAME.to_string(),
            crate::stage_result::path_relative_to_repo(
                context.paths.repository,
                &artifacts.orchestration_manifest,
            ),
            "report_output".to_string(),
        ),
        (
            "source_impute_smoke_metrics_json".to_string(),
            DEFAULT_OUTPUT_SOURCE_SMOKE_NAME.to_string(),
            crate::stage_result::path_relative_to_repo(
                context.paths.repository,
                &artifacts.source_impute_smoke_metrics,
            ),
            "report_output".to_string(),
        ),
    ]
    .into_iter()
    .map(|(artifact_id, declared_path, realized_path, role)| BenchStageResultOutputV1 {
        artifact_id,
        declared_path,
        realized_path,
        role,
        optional: false,
        exists: true,
    })
    .collect()
}
