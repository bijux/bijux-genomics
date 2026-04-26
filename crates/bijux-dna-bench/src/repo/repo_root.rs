use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

fn looks_like_repo_root(path: &Path) -> bool {
    path.join("Cargo.lock").is_file()
        && path.join("crates").is_dir()
        && bijux_dna_infra::configs_dir(path).is_dir()
}

pub fn resolve_repo_root() -> Result<PathBuf> {
    if let Some(path) =
        std::env::var_os("BIJUX_REPO_ROOT").filter(|value| !value.is_empty()).map(PathBuf::from)
    {
        return validate_repo_root_override(path);
    }

    let cwd = std::env::current_dir().map_err(|err| anyhow!("resolve current dir: {err}"))?;
    for candidate in cwd.ancestors() {
        if looks_like_repo_root(candidate) {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(anyhow!(
        "BIJUX_REPO_ROOT must be declared when benchmark loading needs repository contracts"
    ))
}

fn validate_repo_root_override(path: PathBuf) -> Result<PathBuf> {
    if looks_like_repo_root(&path) {
        return Ok(path);
    }
    Err(anyhow!("BIJUX_REPO_ROOT does not point to a bijux repository root: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::validate_repo_root_override;

    #[test]
    fn repo_root_override_must_point_to_repo_root() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;

        let result = validate_repo_root_override(temp.path().to_path_buf());

        assert!(result.is_err());
        let message = result.err().map(|err| err.to_string()).unwrap_or_default();
        assert!(message.contains("does not point to a bijux repository root"));
        Ok(())
    }
}
