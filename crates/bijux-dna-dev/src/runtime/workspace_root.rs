use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// # Errors
/// Returns an error if the repository root cannot be resolved from the current directory or
/// executable location.
pub(super) fn resolve_workspace_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("resolve current directory")?;
    let executable = std::env::current_exe().ok();
    resolve_workspace_root_from_sources(&current_dir, executable.as_deref())
}

fn resolve_workspace_root_from_sources(
    current_dir: &Path,
    executable: Option<&Path>,
) -> Result<PathBuf> {
    workspace_root_ancestor(current_dir, has_workspace_markers)
        .or_else(|| {
            executable.and_then(|path| workspace_root_ancestor(path, has_workspace_markers))
        })
        .with_context(|| {
            let executable = executable
                .map_or_else(|| "unavailable".to_string(), |path| path.display().to_string());
            format!(
                "resolve bijux-genomics workspace from current directory {} or executable {executable}",
                current_dir.display()
            )
        })
}

fn has_workspace_markers(candidate: &Path) -> bool {
    candidate.join("Cargo.toml").is_file()
        && candidate.join("crates/bijux-dna-dev/Cargo.toml").is_file()
}

fn workspace_root_ancestor(
    start: &Path,
    has_workspace_markers: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    start.ancestors().find(|candidate| has_workspace_markers(candidate)).map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{resolve_workspace_root_from_sources, workspace_root_ancestor};

    #[test]
    fn resolves_checkout_from_nested_directory() {
        let checkout = Path::new("checkout");
        let nested = checkout.join("artifacts/runs");

        let resolved = workspace_root_ancestor(&nested, |candidate| candidate == checkout);

        assert_eq!(resolved.as_deref(), Some(checkout));
    }

    #[test]
    fn rejects_directory_outside_checkout() {
        let directory = Path::new("outside");

        let resolved = workspace_root_ancestor(directory, |_| false);

        assert_eq!(resolved, None);
    }

    #[test]
    fn resolves_checkout_from_executable_when_working_directory_is_external() -> anyhow::Result<()>
    {
        let sandbox = tempfile::tempdir()?;
        let checkout = sandbox.path().join("checkout");
        std::fs::create_dir_all(checkout.join("crates/bijux-dna-dev"))?;
        std::fs::write(checkout.join("Cargo.toml"), "[workspace]\n")?;
        std::fs::write(checkout.join("crates/bijux-dna-dev/Cargo.toml"), "[package]\n")?;
        let external = sandbox.path().join("scheduler-working-directory");
        std::fs::create_dir(&external)?;
        let executable = checkout.join("artifacts/target/debug/bijux-dna-dev");

        let resolved = resolve_workspace_root_from_sources(&external, Some(&executable))?;

        assert_eq!(resolved, checkout);
        Ok(())
    }
}
