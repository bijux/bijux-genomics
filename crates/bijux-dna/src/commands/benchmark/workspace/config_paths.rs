use std::path::{Path, PathBuf};

use super::{BENCHMARK_CONFIG_ENV, DEFAULT_BENCHMARK_CONFIG};

fn benchmark_config_path_env_binding() -> Option<PathBuf> {
    std::env::var_os(BENCHMARK_CONFIG_ENV).filter(|value| !value.is_empty()).map(PathBuf::from)
}

pub(crate) fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn resolve_config_path(cwd: &Path, explicit_path: Option<&Path>, default_rel: &str) -> PathBuf {
    if let Some(path) = explicit_path {
        return absolutize(cwd, path);
    }
    if default_rel == DEFAULT_BENCHMARK_CONFIG {
        if let Some(path) = benchmark_config_path_env_binding() {
            return absolutize(cwd, &path);
        }
    }
    cwd.join(default_rel)
}

pub(crate) fn benchmark_config_path(cwd: &Path, explicit_path: Option<&Path>) -> PathBuf {
    resolve_config_path(cwd, explicit_path, DEFAULT_BENCHMARK_CONFIG)
}

pub(crate) fn benchmark_workspace_config_path(cwd: &Path, explicit_path: Option<&Path>) -> PathBuf {
    benchmark_config_path(cwd, explicit_path)
}

pub(crate) fn benchmark_publication_config_path(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> PathBuf {
    benchmark_config_path(cwd, explicit_path)
}
