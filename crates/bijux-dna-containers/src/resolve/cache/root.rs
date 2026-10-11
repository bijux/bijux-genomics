use std::path::PathBuf;

use crate::resolve::RuntimeKind;

#[must_use]
pub(in crate::resolve) fn cache_dir(runner: RuntimeKind) -> PathBuf {
    let cache_root = base_cache_root();
    match runner {
        RuntimeKind::Local => cache_root.join("bijux").join("local").join("state"),
        RuntimeKind::Docker => cache_root.join("bijux").join("docker").join("images"),
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            cache_root.join("bijux").join("apptainer").join("sif")
        }
    }
}

pub(in crate::resolve) fn reference_cache_dir() -> PathBuf {
    base_cache_root().join("bijux").join("references")
}

fn base_cache_root() -> PathBuf {
    std::env::var_os("BIJUX_CACHE_ROOT")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_CACHE_HOME").filter(|value| !value.is_empty()).map(PathBuf::from)
        })
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map_or_else(|| PathBuf::from("."), PathBuf::from)
                .join(".cache")
        })
}
