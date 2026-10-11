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
use crate::ibd::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_FILTERED_SEGMENTS_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_INPUT_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_MERGED_SEGMENTS_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_METRICS_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_TSV_NAME;
use crate::ibd::model::DEFAULT_PROBE_SOURCE_FILTERED_SEGMENTS_NAME;
use crate::ibd::model::DEFAULT_PROBE_SOURCE_ROH_SEGMENTS_NAME;
use crate::ibd::model::DEFAULT_PROBE_SOURCE_ROH_SUMMARY_NAME;
use crate::ibd::model::DEFAULT_PROBE_SOURCE_SUMMARY_NAME;
use crate::ibd::publication::stage_result_output;
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalVcfIbdSmokeReport as LocalVcfIbdSmokeReport;
use std::path::Path;

pub(crate) fn stage_result(
    repo_root: &Path,
    report: &LocalVcfIbdSmokeReport,
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
    report: &LocalVcfIbdSmokeReport,
) -> Vec<crate::execution::stage_result::BenchStageResultOutputV1> {
    vec![
        stage_result_output(
            repo_root,
            "ibd_tsv",
            DEFAULT_OUTPUT_TSV_NAME,
            &repo_root.join(&report.ibd_tsv_path),
            "table_output",
        ),
        stage_result_output(
            repo_root,
            "ibd_json",
            DEFAULT_OUTPUT_JSON_NAME,
            &repo_root.join(&report.ibd_json_path),
            "report_output",
        ),
        stage_result_output(
            repo_root,
            "source_ibd_input_tsv",
            DEFAULT_OUTPUT_SOURCE_INPUT_NAME,
            &repo_root.join(&report.source_ibd_input_path),
            "table_output",
        ),
        stage_result_output(
            repo_root,
            "source_ibd_segments_tsv",
            DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME,
            &repo_root.join(&report.source_ibd_segments_path),
            "table_output",
        ),
        stage_result_output(
            repo_root,
            "source_ibd_merged_segments_tsv",
            DEFAULT_OUTPUT_SOURCE_MERGED_SEGMENTS_NAME,
            &repo_root.join(&report.source_ibd_merged_segments_path),
            "table_output",
        ),
        stage_result_output(
            repo_root,
            "source_ibd_filtered_segments_tsv",
            DEFAULT_OUTPUT_SOURCE_FILTERED_SEGMENTS_NAME,
            &repo_root.join(&report.source_ibd_filtered_segments_path),
            "table_output",
        ),
        stage_result_output(
            repo_root,
            "source_ibd_summary_json",
            DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME,
            &repo_root.join(&report.source_ibd_summary_path),
            "report_output",
        ),
        stage_result_output(
            repo_root,
            "source_ibd_metrics_json",
            DEFAULT_OUTPUT_SOURCE_METRICS_NAME,
            &repo_root.join(&report.source_ibd_metrics_path),
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
            "probe_source_ibd_summary_json",
            DEFAULT_PROBE_SOURCE_SUMMARY_NAME,
            &repo_root.join(&report.insufficient_overlap_probe.source_ibd_summary_path),
            "report_output",
        ),
        stage_result_output(
            repo_root,
            "probe_source_ibd_filtered_segments_tsv",
            DEFAULT_PROBE_SOURCE_FILTERED_SEGMENTS_NAME,
            &repo_root.join(&report.insufficient_overlap_probe.source_ibd_filtered_segments_path),
            "table_output",
        ),
        stage_result_output(
            repo_root,
            "probe_source_roh_summary_json",
            DEFAULT_PROBE_SOURCE_ROH_SUMMARY_NAME,
            &repo_root.join(&report.insufficient_overlap_probe.source_roh_summary_path),
            "report_output",
        ),
        stage_result_output(
            repo_root,
            "probe_source_roh_segments_tsv",
            DEFAULT_PROBE_SOURCE_ROH_SEGMENTS_NAME,
            &repo_root.join(&report.insufficient_overlap_probe.source_roh_segments_path),
            "table_output",
        ),
    ]
}
