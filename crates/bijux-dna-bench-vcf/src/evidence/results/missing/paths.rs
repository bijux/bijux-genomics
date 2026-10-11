use bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow;
use std::path::{Path, PathBuf};
pub(super) fn fixture_result_root(
    fake_result_root: &Path,
    row: &VcfExpectedBenchmarkResultRow,
) -> PathBuf {
    fake_result_root
        .join(&row.corpus_id)
        .join(&row.stage_id)
        .join(&row.asset_profile_id)
        .join(&row.tool_id)
}
pub(super) fn fixture_manifest_path(
    fake_result_root: &Path,
    row: &VcfExpectedBenchmarkResultRow,
) -> PathBuf {
    fixture_result_root(fake_result_root, row).join("stage-result.json")
}
pub(super) fn result_row_id(row: &VcfExpectedBenchmarkResultRow) -> String {
    bijux_dna_core::ids::build_asset_profile_benchmark_result_id(
        &row.domain,
        &row.corpus_id,
        &row.stage_id,
        &row.asset_profile_id,
        &row.tool_id,
    )
}
pub(super) fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}
pub(super) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
