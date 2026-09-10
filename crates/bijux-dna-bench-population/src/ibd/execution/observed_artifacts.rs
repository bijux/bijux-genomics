//! Owner: bijux-dna-bench-population
//! Materialize the exact IBD output artifacts used by report admission.
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_FILTERED_SEGMENTS_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_INPUT_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_MERGED_SEGMENTS_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_METRICS_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME;
use crate::ibd::model::DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) struct IbdArtifacts {
    pub(crate) input: std::path::PathBuf,
    pub(crate) segments: std::path::PathBuf,
    pub(crate) merged_segments: std::path::PathBuf,
    pub(crate) filtered_segments: std::path::PathBuf,
    pub(crate) summary: std::path::PathBuf,
    pub(crate) metrics: std::path::PathBuf,
    pub(crate) logs: std::path::PathBuf,
}

pub(crate) fn copy_artifacts(
    stage_outputs: &bijux_dna_stages_vcf::pipeline::IbdStageOutputs,
    output_root: &Path,
) -> Result<IbdArtifacts> {
    let source_ibd_input_path = output_root.join(DEFAULT_OUTPUT_SOURCE_INPUT_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.ibd_input_tsv, &source_ibd_input_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.ibd_input_tsv.display(),
                source_ibd_input_path.display()
            )
        },
    )?;
    let source_ibd_segments_path = output_root.join(DEFAULT_OUTPUT_SOURCE_SEGMENTS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.ibd_segments_tsv, &source_ibd_segments_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.ibd_segments_tsv.display(),
                source_ibd_segments_path.display()
            )
        })?;
    let source_ibd_merged_segments_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_MERGED_SEGMENTS_NAME);
    bijux_dna_infra::copy_file(
        &stage_outputs.ibd_merged_segments_tsv,
        &source_ibd_merged_segments_path,
    )
    .with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.ibd_merged_segments_tsv.display(),
            source_ibd_merged_segments_path.display()
        )
    })?;
    let source_ibd_filtered_segments_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_FILTERED_SEGMENTS_NAME);
    bijux_dna_infra::copy_file(
        &stage_outputs.ibd_filtered_segments_tsv,
        &source_ibd_filtered_segments_path,
    )
    .with_context(|| {
        format!(
            "copy {} to {}",
            stage_outputs.ibd_filtered_segments_tsv.display(),
            source_ibd_filtered_segments_path.display()
        )
    })?;
    let source_ibd_summary_path = output_root.join(DEFAULT_OUTPUT_SOURCE_SUMMARY_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.ibd_summary_json, &source_ibd_summary_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.ibd_summary_json.display(),
                source_ibd_summary_path.display()
            )
        })?;
    let source_ibd_metrics_path = output_root.join(DEFAULT_OUTPUT_SOURCE_METRICS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.ibd_metrics_json, &source_ibd_metrics_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.ibd_metrics_json.display(),
                source_ibd_metrics_path.display()
            )
        })?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    Ok(IbdArtifacts {
        input: source_ibd_input_path,
        segments: source_ibd_segments_path,
        merged_segments: source_ibd_merged_segments_path,
        filtered_segments: source_ibd_filtered_segments_path,
        summary: source_ibd_summary_path,
        metrics: source_ibd_metrics_path,
        logs: source_logs_path,
    })
}
