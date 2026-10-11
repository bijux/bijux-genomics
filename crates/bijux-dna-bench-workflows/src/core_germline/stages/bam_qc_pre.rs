//! Owner: bijux-dna-bench-workflows
//! Core germline stages bam qc pre.
use super::super::{
    path_relative_to_repo, required_output, BTreeMap, Context, CoreGermlineMicroPipelineRow, Path,
    Result, Value, BAM_QC_PRE_TOOL_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_bam_qc_pre_stage(
    repo_root: &Path,
    output_root: &Path,
    align_row: &CoreGermlineMicroPipelineRow,
    validate_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.qc_pre");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_qc_pre(&aligned_bam)?;
    let report_path = stage_root.join("qc_pre.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::BAM_QC_PRE.to_string(),
        domain: "bam".to_string(),
        tool_id: BAM_QC_PRE_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: summary.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("aligned_bai".to_string(), required_output(align_row, "aligned_bai")?),
            (
                "bam_validation_report".to_string(),
                required_output(validate_row, "bam_validation_report")?,
            ),
        ]),
        outputs: BTreeMap::from([(
            "qc_pre_report".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        metrics: BTreeMap::from([
            ("total_reads".to_string(), Value::from(summary.total_reads)),
            ("mapped_reads".to_string(), Value::from(summary.mapped_reads)),
            ("unmapped_reads".to_string(), Value::from(summary.unmapped_reads)),
            ("duplicate_flagged_reads".to_string(), Value::from(summary.duplicate_flagged_reads)),
        ]),
    })
}
