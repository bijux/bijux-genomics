use crate::readiness::declarations::OutputContext;
use std::path::Path;
use std::path::PathBuf;
pub(super) fn path_template_root(
    context: &OutputContext<'_>,
    stage_id: &str,
    tool_id: &str,
) -> String {
    format!(
        "artifacts/runs/bench/slurm-dry-run/runs/{}/{}/{}/{}/{}",
        context.run_id, "{fixture_scope}", stage_id, "{sample_scope}", tool_id
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
