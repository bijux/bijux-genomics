use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

fn looks_like_repo_root(path: &Path) -> bool {
    path.join("Cargo.lock").is_file()
        && path.join("crates").is_dir()
        && bijux_dna_infra::configs_dir(path).is_dir()
}

fn validate_repo_root(path: PathBuf, source: &str) -> Result<PathBuf> {
    if looks_like_repo_root(&path) {
        Ok(path)
    } else {
        Err(anyhow!(
            "{source} does not identify a Bijux Genomics repository root: {}",
            path.display()
        ))
    }
}

pub(crate) fn resolve_repo_root() -> Result<PathBuf> {
    if let Some(path) =
        std::env::var_os("BIJUX_REPO_ROOT").filter(|value| !value.is_empty()).map(PathBuf::from)
    {
        return validate_repo_root(path, "BIJUX_REPO_ROOT");
    }

    let cwd = std::env::current_dir().map_err(|err| anyhow!("resolve current dir: {err}"))?;
    for candidate in cwd.ancestors() {
        if looks_like_repo_root(candidate) {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(anyhow!(
        "BIJUX_REPO_ROOT must be declared when command resolution needs repository contracts"
    ))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use anyhow::{bail, Result};

    use super::{resolve_repo_root, validate_repo_root};

    fn rust_sources_below(root: &Path, sources: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in fs::read_dir(root)? {
            let path = entry?.path();
            if path.is_dir() {
                rust_sources_below(&path, sources)?;
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                sources.push(path);
            }
        }
        Ok(())
    }

    #[test]
    fn declared_root_must_contain_repository_contracts() -> Result<()> {
        let root = tempfile::tempdir()?;
        let Err(error) = validate_repo_root(root.path().to_path_buf(), "test root") else {
            bail!("incomplete repository root was accepted");
        };

        assert!(error.to_string().contains("does not identify a Bijux Genomics repository root"));
        Ok(())
    }

    #[test]
    fn benchmark_tests_do_not_embed_the_archive_build_root() -> Result<()> {
        let benchmark_root = resolve_repo_root()?.join("crates/bijux-dna/src/commands/benchmark");
        let mut sources = Vec::new();
        rust_sources_below(&benchmark_root, &mut sources)?;

        let forbidden = concat!("env!(", "\"CARGO_MANIFEST_DIR\")");
        let offenders = sources
            .into_iter()
            .filter(|path| fs::read_to_string(path).is_ok_and(|source| source.contains(forbidden)))
            .collect::<Vec<_>>();

        assert!(
            offenders.is_empty(),
            "benchmark tests must resolve the validated runtime repository authority: {offenders:?}"
        );
        Ok(())
    }
}
