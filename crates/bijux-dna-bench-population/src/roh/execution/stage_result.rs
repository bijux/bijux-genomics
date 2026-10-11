//! Owner: bijux-dna-bench-population
//! Project normalized observations into the persisted stage-result contract.
use crate::execution::stage_result::validate_stage_result_manifest;
use crate::execution::stage_result::BenchStageResultCommandV1;
use crate::execution::stage_result::BenchStageResultManifestV1;
use crate::execution::stage_result::BenchStageResultResourceMetricSource;
use crate::execution::stage_result::BenchStageResultResourceMetricsV1;
use crate::execution::stage_result::BenchStageResultRuntimeV1;
use crate::execution::stage_result::BenchStageResultStatus;
use crate::execution::stage_result::BenchStageResultToolV1;
use crate::execution::stage_result::BENCH_STAGE_RESULT_SCHEMA_VERSION;
use crate::roh::model::LocalVcfRohSmokeReport;
use crate::roh::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_METRICS_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_PER_SAMPLE_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_ROH_METRICS_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_ROH_REPORT_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME;
use crate::roh::model::DEFAULT_OUTPUT_TSV_NAME;
use anyhow::Result;
use std::path::Path;

pub(crate) fn stage_result(
    _repo_root: &Path,
    report: &LocalVcfRohSmokeReport,
) -> Result<BenchStageResultManifestV1> {
    let stage_result_manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: report.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: report.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: report.command.clone() },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: report.started_at.clone(),
            finished_at: report.finished_at.clone(),
            elapsed_seconds: report.elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: vec![
            crate::execution::stage_result::ArtifactDeclaration {
                id: "roh_tsv",
                path: DEFAULT_OUTPUT_TSV_NAME,
                role: "table_output",
            }
            .observe(report.roh_tsv_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "roh_json",
                path: DEFAULT_OUTPUT_JSON_NAME,
                role: "report_output",
            }
            .observe(report.roh_json_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "roh_hom",
                path: DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME,
                role: "table_output",
            }
            .observe(report.source_roh_segments_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "roh_report",
                path: DEFAULT_OUTPUT_SOURCE_ROH_REPORT_NAME,
                role: "report_output",
            }
            .observe(report.source_roh_report_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "logs_txt",
                path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
                role: "log_output",
            }
            .observe(report.source_logs_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "source_roh_segments_tsv",
                path: DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME,
                role: "table_output",
            }
            .observe(report.source_roh_segments_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "source_roh_per_sample_tsv",
                path: DEFAULT_OUTPUT_SOURCE_PER_SAMPLE_NAME,
                role: "table_output",
            }
            .observe(report.source_roh_per_sample_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "source_roh_report_json",
                path: DEFAULT_OUTPUT_SOURCE_ROH_REPORT_NAME,
                role: "report_output",
            }
            .observe(report.source_roh_report_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "source_metrics_json",
                path: DEFAULT_OUTPUT_SOURCE_METRICS_NAME,
                role: "report_output",
            }
            .observe(report.source_metrics_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "source_roh_summary_json",
                path: DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME,
                role: "report_output",
            }
            .observe(report.source_roh_summary_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "source_roh_metrics_json",
                path: DEFAULT_OUTPUT_SOURCE_ROH_METRICS_NAME,
                role: "report_output",
            }
            .observe(report.source_roh_metrics_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "source_logs_txt",
                path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
                role: "log_output",
            }
            .observe(report.source_logs_path.clone(), true),
        ],
    };
    validate_stage_result_manifest(&stage_result_manifest)?;
    Ok(stage_result_manifest)
}
