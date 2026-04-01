use std::path::{Path, PathBuf};

use crate::commands::benchmark_workspace::load_optional_benchmark_workspace_config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BenchmarkEnvRoots {
    pub(super) cache_root: PathBuf,
    pub(super) containers_root: PathBuf,
    pub(super) corpus_root: PathBuf,
    pub(super) results_root: PathBuf,
}

pub(super) fn shared_cache_root(root: &Path) -> PathBuf {
    if root
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == ".cache")
    {
        return root.to_path_buf();
    }
    root.join(".cache")
}

pub(super) fn benchmark_env_roots(cwd: &Path, hpc_root: &Path) -> Result<BenchmarkEnvRoots> {
    if let Some(contract) = load_optional_benchmark_workspace_config(cwd, None)? {
        if let Some(remote) = contract.remote {
            if let (
                Some(cache_root),
                Some(containers_root),
                Some(corpus_root),
                Some(results_root),
            ) = (
                remote.cache_root,
                remote.containers_root,
                remote.corpus_root,
                remote.results_root,
            ) {
                return Ok(BenchmarkEnvRoots {
                    cache_root: PathBuf::from(cache_root),
                    containers_root: PathBuf::from(containers_root),
                    corpus_root: PathBuf::from(corpus_root),
                    results_root: PathBuf::from(results_root),
                });
            }
        }
    }
    let cache_root = shared_cache_root(hpc_root);
    let corpus_dir_name = benchmark_fallback_corpus_dir_name();
    Ok(BenchmarkEnvRoots {
        containers_root: cache_root.join("bijux-dna-container"),
        corpus_root: cache_root.join(corpus_dir_name),
        results_root: cache_root.join("results"),
        cache_root,
    })
}

fn benchmark_fallback_corpus_dir_name() -> String {
    std::env::var("BIJUX_BENCHMARK_CORPUS_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("BIJUX_BENCHMARK_CORPUS_ID")
                .ok()
                .map(|value| value.replace('-', "_"))
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "corpus".to_string())
}
