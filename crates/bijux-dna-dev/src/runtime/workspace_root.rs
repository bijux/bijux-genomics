use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// # Errors
/// Returns an error if the repository root cannot be resolved from the current directory.
pub(super) fn resolve_workspace_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("resolve current directory")?;
    resolve_workspace_root_from(&current_dir)
}

fn resolve_workspace_root_from(start: &Path) -> Result<PathBuf> {
    workspace_root_ancestor(start, |candidate| {
        candidate.join("Cargo.toml").is_file()
            && candidate.join("crates/bijux-dna-dev/Cargo.toml").is_file()
    })
    .with_context(|| format!("resolve bijux-genomics workspace above {}", start.display()))
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

    use super::workspace_root_ancestor;

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
}
