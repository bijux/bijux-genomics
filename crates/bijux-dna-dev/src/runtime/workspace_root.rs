use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// # Errors
/// Returns an error if the repository root cannot be resolved from the current directory.
pub(super) fn resolve_workspace_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("resolve current directory")?;
    resolve_workspace_root_from(&current_dir)
}

fn resolve_workspace_root_from(start: &Path) -> Result<PathBuf> {
    start
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && candidate.join("crates/bijux-dna-dev/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .with_context(|| format!("resolve bijux-genomics workspace above {}", start.display()))
}

#[cfg(test)]
mod tests {
    use super::resolve_workspace_root_from;

    #[test]
    fn resolves_checkout_from_nested_directory() -> anyhow::Result<()> {
        let checkout = tempfile::tempdir()?;
        std::fs::write(checkout.path().join("Cargo.toml"), "")?;
        let crate_dir = checkout.path().join("crates/bijux-dna-dev");
        std::fs::create_dir_all(&crate_dir)?;
        std::fs::write(crate_dir.join("Cargo.toml"), "")?;
        let nested = checkout.path().join("artifacts/runs");
        std::fs::create_dir_all(&nested)?;

        let resolved = resolve_workspace_root_from(&nested)?;

        assert_eq!(resolved, checkout.path());
        Ok(())
    }

    #[test]
    fn rejects_directory_outside_checkout() -> anyhow::Result<()> {
        let directory = tempfile::tempdir()?;
        let Err(error) = resolve_workspace_root_from(directory.path()) else {
            anyhow::bail!("directory without workspace markers resolved successfully");
        };

        assert!(error.to_string().contains("resolve bijux-genomics workspace"));
        Ok(())
    }
}
