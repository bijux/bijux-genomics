//! Owner: bijux-dna-bench-population
//! Validate the localized failure probe and its productive control.
use crate::execution::stage_result::path_relative_to_repo;
use crate::ibd::execution::artifacts::read_json;
use crate::ibd::model::DEFAULT_PROBE_INPUT_VCF_NAME;
use crate::ibd::model::DEFAULT_PROBE_SOURCE_FILTERED_SEGMENTS_NAME;
use crate::ibd::model::DEFAULT_PROBE_SOURCE_ROH_SEGMENTS_NAME;
use crate::ibd::model::DEFAULT_PROBE_SOURCE_ROH_SUMMARY_NAME;
use crate::ibd::model::DEFAULT_PROBE_SOURCE_SUMMARY_NAME;
use crate::ibd::observations::records::compute_pair_overlap_marker_counts;
use crate::ibd::observations::records::ordered_pair;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalVcfIbdInsufficientOverlapProbe as LocalVcfIbdInsufficientOverlapProbe;
use bijux_dna_infra::{copy_file, ensure_dir};
use std::path::Path;

pub(crate) fn run_insufficient_overlap_probe(
    repo_root: &Path,
    probe_root: &Path,
    sample_a: &str,
    sample_b: &str,
    tool_id: &str,
) -> Result<LocalVcfIbdInsufficientOverlapProbe> {
    let input_vcf_path = probe_root.join(DEFAULT_PROBE_INPUT_VCF_NAME);
    write_probe_sparse_overlap_vcf(&input_vcf_path, sample_a, sample_b)?;
    let ibd_stage_root = probe_root.join("ibd_stage");
    let roh_stage_root = probe_root.join("roh_stage");
    ensure_dir(&ibd_stage_root).with_context(|| format!("create {}", ibd_stage_root.display()))?;
    ensure_dir(&roh_stage_root).with_context(|| format!("create {}", roh_stage_root.display()))?;

    let super::probe_execution::ProbeOutputs { ibd: ibd_outputs, roh: roh_outputs } =
        super::probe_execution::execute_probe(
            &input_vcf_path,
            &ibd_stage_root,
            &roh_stage_root,
            tool_id,
        )?;

    let source_ibd_summary_path = probe_root.join(DEFAULT_PROBE_SOURCE_SUMMARY_NAME);
    copy_file(&ibd_outputs.ibd_summary_json, &source_ibd_summary_path).with_context(|| {
        format!(
            "copy {} to {}",
            ibd_outputs.ibd_summary_json.display(),
            source_ibd_summary_path.display()
        )
    })?;
    let source_ibd_filtered_segments_path =
        probe_root.join(DEFAULT_PROBE_SOURCE_FILTERED_SEGMENTS_NAME);
    copy_file(&ibd_outputs.ibd_filtered_segments_tsv, &source_ibd_filtered_segments_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                ibd_outputs.ibd_filtered_segments_tsv.display(),
                source_ibd_filtered_segments_path.display()
            )
        })?;
    let source_roh_summary_path = probe_root.join(DEFAULT_PROBE_SOURCE_ROH_SUMMARY_NAME);
    copy_file(&roh_outputs.roh_summary_json, &source_roh_summary_path).with_context(|| {
        format!(
            "copy {} to {}",
            roh_outputs.roh_summary_json.display(),
            source_roh_summary_path.display()
        )
    })?;
    let source_roh_segments_path = probe_root.join(DEFAULT_PROBE_SOURCE_ROH_SEGMENTS_NAME);
    copy_file(&roh_outputs.roh_segments_tsv, &source_roh_segments_path).with_context(|| {
        format!(
            "copy {} to {}",
            roh_outputs.roh_segments_tsv.display(),
            source_roh_segments_path.display()
        )
    })?;

    let ibd_summary = read_json(&source_ibd_summary_path)?;
    let ibd_status = ibd_summary
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("sparse-overlap IBD summary missing `status`"))?
        .to_string();
    if ibd_status != "insufficient_marker_overlap" {
        bail!(
            "sparse-overlap IBD probe expected `insufficient_marker_overlap`, found `{ibd_status}`"
        );
    }
    let insufficient_data_reason = ibd_summary
        .get("insufficient_data_reason")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("sparse-overlap IBD summary missing `insufficient_data_reason`"))?
        .to_string();
    let filtered_segment_count = ibd_summary
        .get("segments_filtered")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("sparse-overlap IBD summary missing `segments_filtered`"))?;
    if filtered_segment_count != 0 {
        bail!("sparse-overlap IBD probe expected zero filtered segments, found {filtered_segment_count}");
    }

    let overlap_counts = compute_pair_overlap_marker_counts(
        &input_vcf_path,
        &[sample_a.to_string(), sample_b.to_string()],
    )?;
    let overlap_marker_count =
        overlap_counts.get(&ordered_pair(sample_a, sample_b)).copied().unwrap_or(0);

    let unrelated_stage_segment_count =
        super::super::observations::probe::validate_productive_roh_control(
            &source_roh_summary_path,
        )?;

    Ok(LocalVcfIbdInsufficientOverlapProbe {
        input_vcf_path: path_relative_to_repo(repo_root, &input_vcf_path),
        ibd_status,
        insufficient_data_reason,
        overlap_marker_count,
        filtered_segment_count,
        source_ibd_summary_path: path_relative_to_repo(repo_root, &source_ibd_summary_path),
        source_ibd_filtered_segments_path: path_relative_to_repo(
            repo_root,
            &source_ibd_filtered_segments_path,
        ),
        unrelated_stage_id: bijux_dna_core::id_catalog::VCF_ROH.to_string(),
        unrelated_stage_status: "complete".to_string(),
        unrelated_stage_segment_count,
        source_roh_summary_path: path_relative_to_repo(repo_root, &source_roh_summary_path),
        source_roh_segments_path: path_relative_to_repo(repo_root, &source_roh_segments_path),
    })
}

pub(crate) fn write_probe_sparse_overlap_vcf(
    path: &Path,
    sample_a: &str,
    sample_b: &str,
) -> Result<()> {
    let raw = format!(
        "##fileformat=VCFv4.2\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\t{sample_a}\t{sample_b}\nchr1\t100\t.\tA\tG\t60\tPASS\t.\tGT\t0/1\t0/0\nchr1\t200\t.\tC\tT\t60\tPASS\t.\tGT\t0/1\t./.\nchr1\t300\t.\tG\tA\t60\tPASS\t.\tGT\t./.\t0/1\n"
    );
    bijux_dna_infra::atomic_write_bytes(path, raw.as_bytes())?;
    Ok(())
}
