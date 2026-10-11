//! Owner: bijux-dna-bench-workflows
//! Core germline stages bam validate.
use super::super::{
    path_relative_to_repo, required_output, BTreeMap, Context, CoreGermlineMicroPipelineRow, Path,
    Result, Value, BAM_VALIDATE_TOOL_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_bam_validate_stage(
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    align_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.validate");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam_path = repo_root.join(required_output(align_row, "aligned_bam")?);
    let aligned_bam_index_path = repo_root.join(required_output(align_row, "aligned_bai")?);
    let report = bijux_dna_domain_bam::execute_bam_validation(
        &aligned_bam_path,
        Some(&aligned_bam_index_path),
        Some(reference_fasta),
    )?;
    let report_path = stage_root.join("validation.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::BAM_VALIDATE.to_string(),
        domain: "bam".to_string(),
        tool_id: BAM_VALIDATE_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("aligned_bai".to_string(), required_output(align_row, "aligned_bai")?),
            (
                "reference_fasta_contract".to_string(),
                path_relative_to_repo(repo_root, reference_fasta),
            ),
        ]),
        outputs: BTreeMap::from([(
            "bam_validation_report".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        metrics: BTreeMap::from([
            (
                "validation_report_present".to_string(),
                Value::from(report.validation_report_present),
            ),
            ("total_reads".to_string(), Value::from(report.flagstat.total_reads.unwrap_or(0))),
            ("mapped_reads".to_string(), Value::from(report.flagstat.mapped_reads.unwrap_or(0))),
            (
                "refusal_code_count".to_string(),
                Value::from(u64::try_from(report.refusal_codes.len()).unwrap_or(u64::MAX)),
            ),
        ]),
    })
}
