use super::ShellSyntaxObservation;
use anyhow::{bail, Context, Result};
use std::path::Path;
pub(super) fn validate_published_commands(
    repo_root: &Path,
    script_path: &Path,
    argv_path: &Path,
    expected_rows: usize,
    inspect_shell: fn(&Path, &Path) -> Result<ShellSyntaxObservation>,
) -> Result<()> {
    let syntax = (inspect_shell)(repo_root, script_path)
        .with_context(|| format!("run bash -n on {}", script_path.display()))?;
    if syntax.exit_code != 0 {
        bail!("VCF rendered commands shell script is not parseable by bash -n:\n{}", syntax.stderr);
    }
    let argv_lines = std::fs::read_to_string(argv_path)
        .with_context(|| format!("read {}", argv_path.display()))?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    if argv_lines != expected_rows {
        bail!(
            "VCF rendered command argv JSONL drifted: expected {expected_rows} rows but found {argv_lines}"
        );
    }
    Ok(())
}
