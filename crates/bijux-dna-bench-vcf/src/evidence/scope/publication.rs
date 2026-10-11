use super::status::{
    count_scope_state, ACTIVE_SCOPE_STATE, COMPLETE_SCOPE_STATE, REMOVED_FROM_SCOPE_STATE,
    VCF_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION,
};
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::scope::{
    VcfActiveStageToolMatrixReport, VcfActiveStageToolMatrixRow,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    path::{Path, PathBuf},
};
/// Publish retained scope after complete projection and contract validation.
///
/// # Errors
/// Returns scope admission, registry acquisition or atomic publication errors.
pub fn render_vcf_active_stage_tool_matrix(
    inputs: &crate::inputs::VcfAdapterInputs,
    repo_root: &Path,
    active: &[bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBindingKey],
    expected: &[bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow],
    output_path: &Path,
) -> Result<VcfActiveStageToolMatrixReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = super::collect_vcf_active_stage_tool_matrix_rows_from_expected(
        inputs, repo_root, active, expected,
    )?;

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_vcf_active_stage_tool_matrix_tsv(&rows).as_bytes(),
    )?;

    let mut scope_state_counts = BTreeMap::<String, usize>::new();
    let mut scope_detail_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *scope_state_counts.entry(row.scope_state.clone()).or_default() += 1;
        *scope_detail_counts.entry(row.scope_detail.clone()).or_default() += 1;
    }

    Ok(VcfActiveStageToolMatrixReport {
        schema_version: VCF_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count: rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len(),
        tool_count: rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len(),
        active_row_count: count_scope_state(&rows, ACTIVE_SCOPE_STATE),
        complete_row_count: count_scope_state(&rows, COMPLETE_SCOPE_STATE),
        removed_row_count: count_scope_state(&rows, REMOVED_FROM_SCOPE_STATE),
        scope_state_counts,
        scope_detail_counts,
        rows,
    })
}
fn render_vcf_active_stage_tool_matrix_tsv(rows: &[VcfActiveStageToolMatrixRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\ttool_status\tstage_support_status\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tscope_state\tscope_detail\tscope_proof_path\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.tool_status),
            sanitize_tsv(&row.stage_support_status),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.parser_id),
            sanitize_tsv(&row.schema_id),
            sanitize_tsv(&row.scope_state),
            sanitize_tsv(&row.scope_detail),
            sanitize_tsv(&row.scope_proof_path),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}
fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n'], " ")
}
