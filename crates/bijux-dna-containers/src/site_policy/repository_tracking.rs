use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use super::validation::safe_checkout_path;

/// List tracked repository files beneath one safe repository-relative prefix.
///
/// The returned paths are sorted, repository-relative identities. Untracked files are excluded so
/// callers can derive governed publications without allowing incidental workspace files to alter
/// the result.
///
/// # Errors
/// Returns an error when the repository or prefix is invalid, Git fails, or a returned path is not
/// valid UTF-8 beneath the requested prefix.
pub fn tracked_repository_files_under(
    repository_root: &Path,
    prefix: &Path,
) -> Result<Vec<PathBuf>> {
    if !repository_root.is_dir() {
        bail!("tracked-file repository root is unavailable: {}", repository_root.display());
    }
    let prefix = safe_checkout_path(prefix)?;
    let prefix_text = prefix.to_str().context("tracked-file prefix is not UTF-8")?;
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["ls-files", "-z", "--", prefix_text])
        .output()
        .context("list tracked repository files")?;
    if !output.status.success() {
        bail!("git ls-files failed: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    let mut paths = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
        .map(|value| {
            let value =
                std::str::from_utf8(value).context("tracked repository path is not UTF-8")?;
            let path = safe_checkout_path(Path::new(value))?;
            if !path.starts_with(&prefix) {
                bail!("tracked repository path escapes requested prefix: {}", path.display());
            }
            Ok(path)
        })
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    paths.dedup();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::tracked_repository_files_under;

    #[test]
    fn tracked_files_are_sorted_and_prefix_scoped() -> anyhow::Result<()> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = tracked_repository_files_under(&root, Path::new("containers/contracts"))?;
        assert!(!paths.is_empty());
        assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(paths.iter().all(|path| path.starts_with("containers/contracts")));
        Ok(())
    }

    #[test]
    fn tracked_files_reject_escaping_prefix() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let error = match tracked_repository_files_under(&root, Path::new("../outside")) {
            Ok(paths) => panic!("escaping prefix unexpectedly returned {paths:?}"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("source freeze path is unsafe"));
    }
}
