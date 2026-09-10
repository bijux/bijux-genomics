//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages bam validate.
use super::super::{
    path_relative_to_repo, required_output, succeeded_row, AdnaMicroPipelineRow, BTreeMap, Context,
    Path, Result, Value, BAM_VALIDATE_TOOL_ID,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_bam_validate_stage(
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    align_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
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

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::BAM_VALIDATE,
            domain: "bam",
            tool_id: BAM_VALIDATE_TOOL_ID,
            execution_mode: "domain_contract",
        },
        path_relative_to_repo(repo_root, &report_path),
        &report.schema_version,
        BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("aligned_bai".to_string(), required_output(align_row, "aligned_bai")?),
        ]),
        BTreeMap::from([(
            "bam_validation_report".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        BTreeMap::from([
            (
                "validation_report_present".to_string(),
                Value::from(report.validation_report_present),
            ),
            ("mapped_reads".to_string(), Value::from(report.flagstat.mapped_reads.unwrap_or(0))),
        ]),
        "validated the aligned BAM against coordinate sort, index, and reference coherence rules"
            .to_string(),
    ))
}
