//! Owner: bijux-dna-bench-population
//! Project normalized observations into the persisted stage-result contract.
use crate::demography::model::LocalVcfDemographySmokeReport;
use crate::demography::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_CONTRACT_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_METRICS_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_NE_TRAJECTORY_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_UPSTREAM_FILTERED_SEGMENTS_NAME;
use crate::demography::model::DEFAULT_OUTPUT_SOURCE_UPSTREAM_IBD_REPORT_NAME;
use crate::demography::model::DEFAULT_PROBE_INPUT_IBD_NAME;
use crate::demography::model::DEFAULT_PROBE_SOURCE_DEMOGRAPHY_CONTRACT_NAME;
use crate::demography::model::DEFAULT_PROBE_SOURCE_DEMOGRAPHY_METRICS_NAME;
use crate::demography::model::DEFAULT_PROBE_SOURCE_LOGS_NAME;
use crate::demography::model::DEFAULT_PROBE_SOURCE_NE_TRAJECTORY_NAME;
use crate::demography::publication::stage_result_output;
use crate::execution::stage_result::validate_stage_result_manifest;
use crate::execution::stage_result::BenchStageResultCommandV1;
use crate::execution::stage_result::BenchStageResultManifestV1;
use crate::execution::stage_result::BenchStageResultResourceMetricSource;
use crate::execution::stage_result::BenchStageResultResourceMetricsV1;
use crate::execution::stage_result::BenchStageResultRuntimeV1;
use crate::execution::stage_result::BenchStageResultStatus;
use crate::execution::stage_result::BenchStageResultToolV1;
use crate::execution::stage_result::BENCH_STAGE_RESULT_SCHEMA_VERSION;
use anyhow::Result;
use std::path::Path;

pub(crate) fn stage_result(
    repo_root: &Path,
    report: &LocalVcfDemographySmokeReport,
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
        outputs: reported_artifacts(repo_root, report),
    };
    validate_stage_result_manifest(&stage_result_manifest)?;
    Ok(stage_result_manifest)
}

fn reported_artifacts(
    repo_root: &Path,
    report: &LocalVcfDemographySmokeReport,
) -> Vec<crate::execution::stage_result::BenchStageResultOutputV1> {
    vec![
        stage_result_output(
            repo_root,
            "demography_json",
            DEFAULT_OUTPUT_JSON_NAME,
            &repo_root.join(&report.demography_json_path),
            "report_output",
        ),
        stage_result_output(
            repo_root,
            "source_upstream_ibd_report_json",
            DEFAULT_OUTPUT_SOURCE_UPSTREAM_IBD_REPORT_NAME,
            &repo_root.join(&report.source_upstream_ibd_report_path),
            "report_output",
        ),
        stage_result_output(
            repo_root,
            "source_upstream_ibd_filtered_segments_tsv",
            DEFAULT_OUTPUT_SOURCE_UPSTREAM_FILTERED_SEGMENTS_NAME,
            &repo_root.join(&report.source_upstream_filtered_segments_path),
            "table_output",
        ),
        stage_result_output(
            repo_root,
            "source_ne_trajectory_tsv",
            DEFAULT_OUTPUT_SOURCE_NE_TRAJECTORY_NAME,
            &repo_root.join(&report.source_ne_trajectory_path),
            "table_output",
        ),
        stage_result_output(
            repo_root,
            "source_demography_contract_json",
            DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_CONTRACT_NAME,
            &repo_root.join(&report.source_demography_contract_path),
            "report_output",
        ),
        stage_result_output(
            repo_root,
            "source_demography_metrics_json",
            DEFAULT_OUTPUT_SOURCE_DEMOGRAPHY_METRICS_NAME,
            &repo_root.join(&report.source_demography_metrics_path),
            "report_output",
        ),
        stage_result_output(
            repo_root,
            "source_logs_txt",
            DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
            &repo_root.join(&report.source_logs_path),
            "log_output",
        ),
        stage_result_output(
            repo_root,
            "probe_input_ibd_segments_tsv",
            DEFAULT_PROBE_INPUT_IBD_NAME,
            &repo_root.join(&report.insufficient_data_probe.input_ibd),
            "table_output",
        ),
        stage_result_output(
            repo_root,
            "probe_source_ne_trajectory_tsv",
            DEFAULT_PROBE_SOURCE_NE_TRAJECTORY_NAME,
            &repo_root.join(&report.insufficient_data_probe.source_ne_trajectory_path),
            "table_output",
        ),
        stage_result_output(
            repo_root,
            "probe_source_demography_contract_json",
            DEFAULT_PROBE_SOURCE_DEMOGRAPHY_CONTRACT_NAME,
            &repo_root.join(&report.insufficient_data_probe.source_demography_contract_path),
            "report_output",
        ),
        stage_result_output(
            repo_root,
            "probe_source_demography_metrics_json",
            DEFAULT_PROBE_SOURCE_DEMOGRAPHY_METRICS_NAME,
            &repo_root.join(&report.insufficient_data_probe.source_demography_metrics_path),
            "report_output",
        ),
        stage_result_output(
            repo_root,
            "probe_source_logs_txt",
            DEFAULT_PROBE_SOURCE_LOGS_NAME,
            &repo_root.join(&report.insufficient_data_probe.source_logs_path),
            "log_output",
        ),
    ]
}
