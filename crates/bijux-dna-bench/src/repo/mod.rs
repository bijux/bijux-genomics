//! Owner: bijux-dna-bench
//! Run repositories for bench.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub mod run_repo;
pub mod sqlite;
mod workspace_paths;

pub use run_repo::RunRepository;
pub use workspace_paths::{bench_data_dir, bench_suites_dir};

fn looks_like_repo_root(path: &Path) -> bool {
    path.join("Cargo.lock").is_file()
        && path.join("crates").is_dir()
        && bijux_dna_infra::configs_dir(path).is_dir()
}

pub fn resolve_repo_root() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("BIJUX_REPO_ROOT")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Ok(path);
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
