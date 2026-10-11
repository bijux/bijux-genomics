//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages bam mapping summary.
use super::super::{
    path_relative_to_repo, required_output, succeeded_row, AdnaMicroPipelineRow, BTreeMap, Context,
    Path, Result, Value, BAM_MAPPING_SUMMARY_TOOL_ID,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_bam_mapping_summary_stage(
    repo_root: &Path,
    output_root: &Path,
    align_row: &AdnaMicroPipelineRow,
    validate_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.mapping_summary");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let align_sam = repo_root.join(required_output(align_row, "align_sam")?);
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_mapping(&align_sam)?;
    let report_path = stage_root.join("mapping_summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::BAM_MAPPING_SUMMARY,
            domain: "bam",
            tool_id: BAM_MAPPING_SUMMARY_TOOL_ID,
            execution_mode: "domain_contract",
        },
        path_relative_to_repo(repo_root, &report_path),
        &summary.schema_version,
        BTreeMap::from([
            ("align_sam".to_string(), required_output(align_row, "align_sam")?),
            (
                "bam_validation_report".to_string(),
                required_output(validate_row, "bam_validation_report")?,
            ),
        ]),
        BTreeMap::from([(
            "mapping_summary_report_json".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        BTreeMap::from([
            ("mapped_reads".to_string(), Value::from(summary.flagstat.mapped_reads.unwrap_or(0))),
            (
                "mapped_fraction".to_string(),
                Value::from(summary.flagstat.mapped_fraction.unwrap_or(0.0)),
            ),
        ]),
        "summarized mapping behavior from the aligned aDNA-like BAM".to_string(),
    ))
}
