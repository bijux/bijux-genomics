//! Owner: bijux-dna-bench-workflows
//! Core germline stages bam align.
use super::super::{
    bail, fs, path_relative_to_repo, required_output, write_mapped_sam_from_fastqs,
    AlignmentFixtureOperations, BTreeMap, Context, CoreGermlineMicroPipelineRow, Path,
    PipelineInputFixtures, ReadGroupSpec, Result, Value, BAM_ALIGN_TOOL_ID, PIPELINE_SAMPLE_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_bam_align_stage(
    alignment: &AlignmentFixtureOperations,
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    input_fixtures: &PipelineInputFixtures,
    filter_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.align");
    let tiny_align_root = stage_root.join("semantic-runtime");
    bijux_dna_infra::ensure_dir(&tiny_align_root)
        .with_context(|| format!("create {}", tiny_align_root.display()))?;

    let filtered_r1 = repo_root.join(required_output(filter_row, "filtered_reads_r1_path")?);
    let filtered_r2 = repo_root.join(required_output(filter_row, "filtered_reads_r2_path")?);
    let read_group = ReadGroupSpec::with_defaults(PIPELINE_SAMPLE_ID);
    let (provenance, mapping_summary) = bijux_dna_domain_bam::align_fastq_to_bam_bowtie2_style(
        reference_fasta,
        &filtered_r1,
        Some(&filtered_r2),
        &tiny_align_root,
        PIPELINE_SAMPLE_ID,
        &read_group,
        Some("very_sensitive_local"),
    )?;

    let semantic_sam_path = tiny_align_root.join("align.bam");
    if !semantic_sam_path.is_file() {
        bail!("semantic BAM alignment runtime did not produce {}", semantic_sam_path.display());
    }

    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let semantic_sam = stage_root.join("align.semantic.sam");
    fs::copy(&semantic_sam_path, &semantic_sam).with_context(|| {
        format!("copy {} to {}", semantic_sam_path.display(), semantic_sam.display())
    })?;
    let aligned_sam_path = stage_root.join("align.sam");
    write_mapped_sam_from_fastqs(
        reference_fasta,
        &filtered_r1,
        &filtered_r2,
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
    let mapping_summary_path = stage_root.join("mapping_summary.json");
    bijux_dna_infra::atomic_write_json(&mapping_summary_path, &mapping_summary)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::BAM_ALIGN.to_string(),
        domain: "bam".to_string(),
        tool_id: BAM_ALIGN_TOOL_ID.to_string(),
        execution_mode: "semantic_alignment".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &provenance_path),
        parsed_schema_version: provenance.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            (
                "filtered_reads_r1_path".to_string(),
                required_output(filter_row, "filtered_reads_r1_path")?,
            ),
            (
                "filtered_reads_r2_path".to_string(),
                required_output(filter_row, "filtered_reads_r2_path")?,
            ),
            (
                "reference_fasta_contract".to_string(),
                path_relative_to_repo(repo_root, reference_fasta),
            ),
            (
                "alignment_read_group_contract".to_string(),
                path_relative_to_repo(repo_root, &read_group_contract),
            ),
        ]),
        outputs: BTreeMap::from([
            ("aligned_bam".to_string(), path_relative_to_repo(repo_root, &aligned_bam_path)),
            ("aligned_bai".to_string(), path_relative_to_repo(repo_root, &aligned_bam_index_path)),
            ("align_metrics".to_string(), path_relative_to_repo(repo_root, &mapping_summary_path)),
            ("align_provenance".to_string(), path_relative_to_repo(repo_root, &provenance_path)),
            ("align_sam".to_string(), path_relative_to_repo(repo_root, &aligned_sam_path)),
            ("align_semantic_sam".to_string(), path_relative_to_repo(repo_root, &semantic_sam)),
        ]),
        metrics: BTreeMap::from([
            (
                "mapped_reads".to_string(),
                Value::from(mapping_summary.flagstat.mapped_reads.unwrap_or(0)),
            ),
            (
                "total_reads".to_string(),
                Value::from(mapping_summary.flagstat.total_reads.unwrap_or(0)),
            ),
            (
                "proper_pair_reads".to_string(),
                Value::from(mapping_summary.proper_pair_reads.unwrap_or(0)),
            ),
        ]),
    })
}
