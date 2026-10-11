use super::super::paths::{path_relative_to_repo, repo_relative_path};
use super::model::FastqActiveStageToolMatrixReport;
use super::serialization::render_fastq_active_stage_tool_matrix_tsv;
use super::{FASTQ_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION, REMOVED_FROM_SCOPE_PROOF_PATH};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Build or publish admitted FASTQ readiness evidence.
///
/// # Errors
/// Returns an error for invalid inputs, incompatible evidence or publication failure.
pub fn render_fastq_active_stage_tool_matrix(
    repo_root: &Path,
    output_path: &Path,
    input: (super::super::inputs::FastqAdapterCoverage, Vec<super::super::inputs::FastqScopeRow>),
) -> Result<FastqActiveStageToolMatrixReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let collection =
        super::projection::collect_fastq_active_stage_tool_matrix_rows(input.0, input.1)?;

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_fastq_active_stage_tool_matrix_tsv(&collection.rows).as_bytes(),
    )
    .with_context(|| format!("write {}", output_path.display()))?;

    let mut support_status_counts = BTreeMap::<String, usize>::new();
    let mut parser_status_counts = BTreeMap::<String, usize>::new();
    let mut corpus_status_counts = BTreeMap::<String, usize>::new();
    for row in &collection.rows {
        *support_status_counts.entry(row.support_status.clone()).or_default() += 1;
        *parser_status_counts.entry(row.parser_status.clone()).or_default() += 1;
        *corpus_status_counts.entry(row.corpus_status.clone()).or_default() += 1;
    }

    Ok(FastqActiveStageToolMatrixReport {
        schema_version: FASTQ_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        retained_row_count: collection.retained_row_count,
        retained_stage_count: collection.retained_stage_count,
        retained_tool_count: collection.retained_tool_count,
        row_count: collection.rows.len(),
        stage_count: collection
            .rows
            .iter()
            .map(|row| row.stage_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        tool_count: collection
            .rows
            .iter()
            .map(|row| row.tool_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        removed_row_count: collection.removed_row_count,
        removed_stage_count: collection.removed_stage_count,
        removed_tool_count: collection.removed_tool_count,
        removed_from_scope_path: REMOVED_FROM_SCOPE_PROOF_PATH,
        support_status_counts,
        parser_status_counts,
        corpus_status_counts,
        rows: collection.rows,
    })
}
