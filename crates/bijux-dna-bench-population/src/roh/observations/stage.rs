//! Owner: bijux-dna-bench-population
//! Observe normalized stage outputs and preserve failure-probe classifications.
use crate::roh::execution::artifacts::read_json;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_METRICS_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_PER_SAMPLE_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_ROH_METRICS_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_ROH_REPORT_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME;
use crate::roh::model::DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME;
use crate::roh::observations::evidence::validate_roh_summary_contract;
use crate::roh::observations::records::parse_roh_per_sample_summary;
use crate::roh::observations::records::parse_roh_segments;
use crate::roh::LocalVcfRohSmokePerSampleSummary;
use crate::roh::LocalVcfRohSmokeSegment;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeSet;

pub(crate) struct ObservedRohStage {
    pub(crate) source_roh_segments_path: std::path::PathBuf,
    pub(crate) source_roh_per_sample_path: std::path::PathBuf,
    pub(crate) source_roh_report_path: std::path::PathBuf,
    pub(crate) source_metrics_path: std::path::PathBuf,
    pub(crate) source_roh_summary_path: std::path::PathBuf,
    pub(crate) source_roh_metrics_path: std::path::PathBuf,
    pub(crate) source_logs_path: std::path::PathBuf,
    pub(crate) segments: Vec<LocalVcfRohSmokeSegment>,
    pub(crate) per_sample_summary: Vec<LocalVcfRohSmokePerSampleSummary>,
    pub(crate) execution_mode: String,
    pub(crate) tool_ok: bool,
    pub(crate) segment_count: u64,
    pub(crate) sample_count: u64,
    pub(crate) total_length: u64,
}

pub(crate) fn observe_stage(
    prepared: &super::super::execution::inputs::PreparedRohInputs,
    stage_outputs: &bijux_dna_stages_vcf::pipeline::RohStageOutputs,
) -> Result<ObservedRohStage> {
    let super::super::execution::inputs::PreparedRohInputs {
        expected_samples, output_root, ..
    } = prepared;
    let source_roh_segments_path = output_root.join(DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.roh_segments_tsv, &source_roh_segments_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.roh_segments_tsv.display(),
                source_roh_segments_path.display()
            )
        })?;
    let source_roh_per_sample_path = output_root.join(DEFAULT_OUTPUT_SOURCE_PER_SAMPLE_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.roh_per_sample_tsv, &source_roh_per_sample_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.roh_per_sample_tsv.display(),
                source_roh_per_sample_path.display()
            )
        })?;
    let source_roh_report_path = output_root.join(DEFAULT_OUTPUT_SOURCE_ROH_REPORT_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.roh_json, &source_roh_report_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.roh_json.display(),
                source_roh_report_path.display()
            )
        },
    )?;
    let source_metrics_path = output_root.join(DEFAULT_OUTPUT_SOURCE_METRICS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.metrics_json, &source_metrics_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.metrics_json.display(),
                source_metrics_path.display()
            )
        },
    )?;
    let source_roh_summary_path = output_root.join(DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.roh_summary_json, &source_roh_summary_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.roh_summary_json.display(),
                source_roh_summary_path.display()
            )
        })?;
    let source_roh_metrics_path = output_root.join(DEFAULT_OUTPUT_SOURCE_ROH_METRICS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.roh_metrics_json, &source_roh_metrics_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.roh_metrics_json.display(),
                source_roh_metrics_path.display()
            )
        })?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let segments = parse_roh_segments(&source_roh_segments_path, &expected_sample_set)?;
    let per_sample_summary =
        parse_roh_per_sample_summary(&source_roh_per_sample_path, expected_samples)?;
    let source_roh_report = read_json(&source_roh_report_path)?;
    validate_roh_summary_contract(&source_roh_report, &segments, &per_sample_summary)?;

    let execution_mode = source_roh_report
        .get("execution_mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("ROH source report missing `execution_mode`"))?
        .to_string();
    let tool_ok = source_roh_report
        .pointer("/tool_attempts/plink2_homozyg")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("ROH source report missing `tool_attempts.plink2_homozyg`"))?;
    let segment_count =
        u64::try_from(segments.len()).map_err(|_| anyhow!("segment count overflow"))?;
    let sample_count =
        u64::try_from(per_sample_summary.len()).map_err(|_| anyhow!("sample count overflow"))?;
    let total_length = segments.iter().map(|segment| segment.length).sum::<u64>();

    Ok(ObservedRohStage {
        source_roh_segments_path,
        source_roh_per_sample_path,
        source_roh_report_path,
        source_metrics_path,
        source_roh_summary_path,
        source_roh_metrics_path,
        source_logs_path,
        segments,
        per_sample_summary,
        execution_mode,
        tool_ok,
        segment_count,
        sample_count,
        total_length,
    })
}
