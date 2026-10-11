//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages bam align.
use super::super::{
    bail, fs, path_relative_to_repo, required_output, succeeded_row, write_mapped_sam_from_fastqs,
    AdnaMicroPipelineRow, AlignmentFixtureOperations, BTreeMap, Context, Path,
    PipelineInputFixtures, ReadGroupSpec, Result, Value, ADNA_PIPELINE_SAMPLE_ID,
    BAM_ALIGN_TOOL_ID,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_bam_align_stage(
    alignment: &AlignmentFixtureOperations,
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    input_fixtures: &PipelineInputFixtures,
    dedup_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.align");
    let tiny_align_root = stage_root.join("semantic-runtime");
    bijux_dna_infra::ensure_dir(&tiny_align_root)
        .with_context(|| format!("create {}", tiny_align_root.display()))?;

    let dedup_r1 = repo_root.join(required_output(dedup_row, "deduplicated_reads_r1_path")?);
    let dedup_r2 = repo_root.join(required_output(dedup_row, "deduplicated_reads_r2_path")?);
    let read_group = ReadGroupSpec::with_defaults(ADNA_PIPELINE_SAMPLE_ID);
    let (provenance, mapping_summary) = bijux_dna_domain_bam::align_fastq_to_bam_bowtie2_style(
        reference_fasta,
        &dedup_r1,
        Some(&dedup_r2),
        &tiny_align_root,
        ADNA_PIPELINE_SAMPLE_ID,
        &read_group,
        Some("very_sensitive_local"),
    )?;

    let semantic_sam_path = tiny_align_root.join("align.bam");
    if !semantic_sam_path.is_file() {
        bail!("semantic BAM alignment runtime did not produce {}", semantic_sam_path.display());
    }

    let semantic_sam = stage_root.join("align.semantic.sam");
    fs::copy(&semantic_sam_path, &semantic_sam).with_context(|| {
        format!("copy {} to {}", semantic_sam_path.display(), semantic_sam.display())
    })?;
    let aligned_sam_path = stage_root.join("align.sam");
    write_mapped_sam_from_fastqs(
        reference_fasta,
        &input_fixtures.reference_name,
        &dedup_r1,
        &dedup_r2,
        &read_group,
        &input_fixtures.placements,
        &aligned_sam_path,
    )?;
    let aligned_bam_path = stage_root.join("align.bam");
    let aligned_bam_index_path = stage_root.join("align.bam.bai");
    alignment.encode_coordinate_bam(
        &aligned_sam_path,
        &aligned_bam_path,
        &aligned_bam_index_path,
    )?;

    let read_group_contract = stage_root.join("read_group.json");
    bijux_dna_infra::atomic_write_json(&read_group_contract, &read_group)?;
    let provenance_path = stage_root.join("alignment.provenance.json");
    bijux_dna_infra::atomic_write_json(&provenance_path, &provenance)?;
    let mapping_summary_path = stage_root.join("alignment.mapping_summary.json");
    bijux_dna_infra::atomic_write_json(&mapping_summary_path, &mapping_summary)?;

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::BAM_ALIGN,
            domain: "bam",
            tool_id: BAM_ALIGN_TOOL_ID,
            execution_mode: "synthetic_alignment",
        },
        path_relative_to_repo(repo_root, &provenance_path),
        &provenance.schema_version,
        BTreeMap::from([
            (
                "deduplicated_reads_r1_path".to_string(),
                required_output(dedup_row, "deduplicated_reads_r1_path")?,
            ),
            (
                "deduplicated_reads_r2_path".to_string(),
                required_output(dedup_row, "deduplicated_reads_r2_path")?,
            ),
            (
                "reference_fasta_contract".to_string(),
                path_relative_to_repo(repo_root, reference_fasta),
            ),
        ]),
        BTreeMap::from([
            ("aligned_bam".to_string(), path_relative_to_repo(repo_root, &aligned_bam_path)),
            ("aligned_bai".to_string(), path_relative_to_repo(repo_root, &aligned_bam_index_path)),
            ("align_sam".to_string(), path_relative_to_repo(repo_root, &aligned_sam_path)),
            ("align_metrics".to_string(), path_relative_to_repo(repo_root, &mapping_summary_path)),
            ("align_provenance".to_string(), path_relative_to_repo(repo_root, &provenance_path)),
        ]),
        BTreeMap::from([
            (
                "mapped_reads".to_string(),
                Value::from(mapping_summary.flagstat.mapped_reads.unwrap_or(0)),
            ),
            (
                "total_reads".to_string(),
                Value::from(mapping_summary.flagstat.total_reads.unwrap_or(0)),
            ),
        ]),
        "aligned duplicate-aware aDNA-like reads against the governed ancient-DNA reference"
            .to_string(),
    ))
}
