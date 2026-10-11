use crate::bam::assignment::declarations::BAM_CORPUS_ASSIGNMENT_SCHEMA_VERSION;
use crate::bam::assignment::models::BamCorpusAssignmentReport;
use crate::bam::assignment::paths::path_relative_to_repo;
use crate::bam::assignment::paths::repo_relative_path;
use crate::bam::assignment::rows::collect_bam_corpus_assignment_rows;
use crate::bam::assignment::tsv::render_bam_corpus_assignment_tsv;
use crate::bam::assignment::AssignmentContext;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
/// Construct the governed BAM corpus assignment evidence.
///
/// # Errors
/// Returns canonical source, fixture identity, scientific contract or publication failure.
pub fn render_bam_corpus_assignment(
    context: &AssignmentContext<'_>,
    output_path: &Path,
) -> Result<BamCorpusAssignmentReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let stage_count = context.inventory.stage_count(repo_root)?;
    let (_, tool_count, rows) = collect_bam_corpus_assignment_rows(context)?;

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let rendered = render_bam_corpus_assignment_tsv(&rows);
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())
        .with_context(|| format!("write {}", output_path.display()))?;

    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let mut corpus_family_counts = BTreeMap::<String, usize>::new();
    let mut fixture_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *corpus_family_counts.entry(row.corpus_family_id.clone()).or_default() += 1;
        *fixture_counts.entry(row.fixture_id.clone()).or_default() += 1;
    }

    Ok(BamCorpusAssignmentReport {
        schema_version: BAM_CORPUS_ASSIGNMENT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        benchmark_ready_row_count,
        corpus_family_counts,
        fixture_counts,
        rows,
    })
}
