use super::serialization::{
    render_vcf_pca_command_argv_jsonl, render_vcf_pca_commands_shell_script,
};
use super::{PreparedPcaAdapterEvidence, VCF_PCA_RENDERED_COMMANDS_PATH};
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::command_reports::VcfRenderedCommandsReport;
use std::path::Path;
/// Publish compatible PCA command artifacts after both representations serialize successfully.
///
/// # Errors
/// Returns command serialization or atomic file publication failure.
pub fn publish_pca_commands(
    repo_root: &Path,
    prepared: &PreparedPcaAdapterEvidence,
) -> Result<VcfRenderedCommandsReport> {
    let output_path = repo_root.join(VCF_PCA_RENDERED_COMMANDS_PATH);
    let argv_output_path = output_path.with_file_name("vcf-pca-rendered-commands.argv.jsonl");
    let rows = prepared.rows.iter().map(|row| row.command.clone()).collect::<Vec<_>>();
    let script = render_vcf_pca_commands_shell_script(
        &rows,
        &repo_root_relative_to_output(repo_root, &output_path),
    );
    let argv = render_vcf_pca_command_argv_jsonl(&rows)?;
    bijux_dna_infra::atomic_write_bytes(&output_path, script.as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(&argv_output_path, argv.as_bytes())?;
    Ok(VcfRenderedCommandsReport {
        schema_version: "bijux.bench.readiness.vcf_rendered_commands.v1",
        output_path: path_relative_to_repo(repo_root, &output_path),
        argv_output_path: path_relative_to_repo(repo_root, &argv_output_path),
        row_count: rows.len(),
        rows,
    })
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
fn repo_root_relative_to_output(repo_root: &Path, output_path: &Path) -> String {
    let relative_output_path = output_path.strip_prefix(repo_root).unwrap_or(output_path);
    let depth = relative_output_path.parent().map_or(0, |parent| parent.components().count());
    if depth == 0 {
        ".".to_string()
    } else {
        std::iter::repeat_n("..", depth).collect::<Vec<_>>().join("/")
    }
}
