use std::path::PathBuf;

pub(super) fn declared_repo_root() -> Option<PathBuf> {
    std::env::var_os("BIJUX_REPO_ROOT")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
