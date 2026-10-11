//! Owner: bijux-dna-bench-population
//! Admit normalized IBD observations and run the localized overlap failure probe.
use crate::ibd::execution::artifacts::read_json;
use crate::ibd::fixtures::failure_probe::run_insufficient_overlap_probe;
use crate::ibd::observations::evidence::summarize_ibd_rows;
use crate::ibd::observations::records::compute_pair_overlap_marker_counts;
use crate::ibd::observations::records::parse_ibd_filtered_segments;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) struct ObservedIbdStage {
    pub(crate) source_ibd_input_path: std::path::PathBuf,
    pub(crate) source_ibd_segments_path: std::path::PathBuf,
    pub(crate) source_ibd_merged_segments_path: std::path::PathBuf,
    pub(crate) source_ibd_filtered_segments_path: std::path::PathBuf,
    pub(crate) source_ibd_summary_path: std::path::PathBuf,
    pub(crate) source_ibd_metrics_path: std::path::PathBuf,
    pub(crate) source_logs_path: std::path::PathBuf,
    pub(crate) rows: Vec<bijux_dna_bench_model::BenchmarkLocalVcfIbdSmokeRow>,
    pub(crate) status: String,
    pub(crate) execution_mode: String,
    pub(crate) germline_tool_ok: bool,
    pub(crate) ibdseq_tool_ok: bool,
    pub(crate) ibdhap_tool_ok: bool,
    pub(crate) insufficient_overlap_probe:
        bijux_dna_bench_model::BenchmarkLocalVcfIbdInsufficientOverlapProbe,
}

pub(crate) fn observe_stage(
    repo_root: &Path,
    prepared: &super::super::execution::inputs::PreparedIbdInputs,
    stage_outputs: &bijux_dna_stages_vcf::pipeline::IbdStageOutputs,
    contract: &crate::ibd::model::GovernedVcfIbdSmokeContract,
) -> Result<ObservedIbdStage> {
    let super::super::execution::inputs::PreparedIbdInputs {
        expected_samples,
        output_root,
        probe_root,
        input_vcf_path,
        ..
    } = prepared;
    let super::super::execution::observed_artifacts::IbdArtifacts {
        input: source_ibd_input_path,
        segments: source_ibd_segments_path,
        merged_segments: source_ibd_merged_segments_path,
        filtered_segments: source_ibd_filtered_segments_path,
        summary: source_ibd_summary_path,
        metrics: source_ibd_metrics_path,
        logs: source_logs_path,
    } = super::super::execution::observed_artifacts::copy_artifacts(stage_outputs, output_root)?;

    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let filtered_segments =
        parse_ibd_filtered_segments(&source_ibd_filtered_segments_path, &expected_sample_set)?;
    let overlap_counts = compute_pair_overlap_marker_counts(input_vcf_path, expected_samples)?;
    let rows = summarize_ibd_rows(&filtered_segments, &overlap_counts)?;
    if rows.is_empty() {
        bail!("governed VCF IBD smoke expected at least one pair row");
    }

    let source_ibd_summary = read_json(&source_ibd_summary_path)?;
    let status = source_ibd_summary
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("IBD source summary missing `status`"))?
        .to_string();
    if status != "complete" {
        bail!("governed VCF IBD smoke expected `complete` status, found `{status}`");
    }
    let execution_mode = source_ibd_summary
        .get("execution_mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("IBD source summary missing `execution_mode`"))?
        .to_string();
    let germline_tool_ok = source_ibd_summary
        .pointer("/tool_attempts/germline")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("IBD source summary missing `tool_attempts.germline`"))?;
    let ibdseq_tool_ok = source_ibd_summary
        .pointer("/tool_attempts/ibdseq")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("IBD source summary missing `tool_attempts.ibdseq`"))?;
    let ibdhap_tool_ok = source_ibd_summary
        .pointer("/tool_attempts/ibdhap")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("IBD source summary missing `tool_attempts.ibdhap`"))?;

    let insufficient_overlap_probe = run_insufficient_overlap_probe(
        repo_root,
        probe_root,
        &expected_samples[0],
        &expected_samples[1],
        &contract.tool,
    )?;

    Ok(ObservedIbdStage {
        source_ibd_input_path,
        source_ibd_segments_path,
        source_ibd_merged_segments_path,
        source_ibd_filtered_segments_path,
        source_ibd_summary_path,
        source_ibd_metrics_path,
        source_logs_path,
        rows,
        status,
        execution_mode,
        germline_tool_ok,
        ibdseq_tool_ok,
        ibdhap_tool_ok,
        insufficient_overlap_probe,
    })
}
