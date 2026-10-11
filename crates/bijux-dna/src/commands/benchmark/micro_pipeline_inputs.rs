use anyhow::Result;
use bijux_dna_bench_workflows::inputs::{AlignmentFixtureOperations, CoreGermlineWorkflowInputs};
use std::path::Path;
pub(super) fn alignment() -> AlignmentFixtureOperations {
    AlignmentFixtureOperations::new(
        bijux_dna_api::v1::api::bench::alignment_fixtures::convert_coordinate_sam_to_bam,
        reference_intervals,
    )
}
fn reference_intervals(
    bam: &Path,
    encoding: bijux_dna_bench_workflows::inputs::AlignmentFixtureFormat,
    bed: &Path,
) -> Result<()> {
    use bijux_dna_api::v1::api::bench::alignment_fixtures::{
        write_alignment_reference_bed, AlignmentInputFormat,
    };
    let format = match encoding {
        bijux_dna_bench_workflows::inputs::AlignmentFixtureFormat::Sam => AlignmentInputFormat::Sam,
        bijux_dna_bench_workflows::inputs::AlignmentFixtureFormat::Bam => AlignmentInputFormat::Bam,
    };
    write_alignment_reference_bed(bam, format, bed)
}
pub(super) fn core_germline() -> Result<CoreGermlineWorkflowInputs> {
    let rows = super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows()?;
    CoreGermlineWorkflowInputs::new(
        alignment(),
        rows.into_iter()
            .filter(|row| {
                ["vcf.call", "vcf.filter", "vcf.stats", "vcf.qc"].contains(&row.stage_id.as_str())
            })
            .map(|row| (row.stage_id, row.tool_id)),
    )
}
