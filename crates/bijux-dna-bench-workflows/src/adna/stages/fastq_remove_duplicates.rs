//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages fastq remove duplicates.
use super::super::{
    path_relative_to_repo, remove_duplicates, required_output, succeeded_row, AdnaMicroPipelineRow,
    BTreeMap, Context, DedupMode, PairedMode, Path, RemoveDuplicatesEffectiveParams, Result, Value,
    FASTQ_REMOVE_DUPLICATES_TOOL_ID, REMOVE_DUPLICATES_SCHEMA_VERSION,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_fastq_remove_duplicates_stage(
    repo_root: &Path,
    output_root: &Path,
    trim_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.remove_duplicates");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let input_r1 =
        repo_root.join(required_output(trim_row, "terminal_damage_trimmed_reads_r1_path")?);
    let input_r2 =
        repo_root.join(required_output(trim_row, "terminal_damage_trimmed_reads_r2_path")?);
    let output_r1 = stage_root.join("deduplicated_R1.fastq");
    let output_r2 = stage_root.join("deduplicated_R2.fastq");
    let classes_path = stage_root.join("duplicate_classes.tsv");
    let provenance_path = stage_root.join("duplicate_provenance.json");
    let report_path = stage_root.join("deduplication_report.json");
    let backend_path = stage_root.join("remove_duplicates.backend.txt");
    let report = remove_duplicates(
        &input_r1,
        Some(&input_r2),
        &RemoveDuplicatesEffectiveParams {
            schema_version: REMOVE_DUPLICATES_SCHEMA_VERSION.to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 1,
            dedup_mode: DedupMode::Exact,
            keep_order: true,
        },
        &output_r1,
        Some(&output_r2),
        &classes_path,
        &provenance_path,
        &report_path,
        Some(&backend_path),
    )?;

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::FASTQ_DEDUPLICATE,
            domain: "fastq",
            tool_id: FASTQ_REMOVE_DUPLICATES_TOOL_ID,
            execution_mode: "domain_contract",
        },
        path_relative_to_repo(repo_root, &report_path),
        &report.schema_version,
        BTreeMap::from([
            (
                "terminal_damage_trimmed_reads_r1_path".to_string(),
                required_output(trim_row, "terminal_damage_trimmed_reads_r1_path")?,
            ),
            (
                "terminal_damage_trimmed_reads_r2_path".to_string(),
                required_output(trim_row, "terminal_damage_trimmed_reads_r2_path")?,
            ),
        ]),
        BTreeMap::from([
            (
                "deduplicated_reads_r1_path".to_string(),
                path_relative_to_repo(repo_root, &output_r1),
            ),
            (
                "deduplicated_reads_r2_path".to_string(),
                path_relative_to_repo(repo_root, &output_r2),
            ),
            ("duplicate_classes_tsv".to_string(), path_relative_to_repo(repo_root, &classes_path)),
            (
                "duplicate_provenance_json".to_string(),
                path_relative_to_repo(repo_root, &provenance_path),
            ),
            ("deduplication_report".to_string(), path_relative_to_repo(repo_root, &report_path)),
        ]),
        BTreeMap::from([
            ("reads_in".to_string(), Value::from(report.reads_in)),
            ("reads_out".to_string(), Value::from(report.reads_out)),
            ("duplicates_removed".to_string(), Value::from(report.duplicates_removed)),
        ]),
        "removed exact duplicate aDNA-like read pairs while preserving first-observed order"
            .to_string(),
    ))
}
