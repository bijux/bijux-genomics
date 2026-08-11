use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Resolve the runtime workspace root, falling back to a crate manifest directory.
///
/// # Panics
/// Panics if `manifest_dir` does not have the expected `crates/<name>` layout.
#[must_use]
pub fn workspace_root_from_manifest(manifest_dir: &str) -> PathBuf {
    let runtime_root = std::env::var_os("BIJUX_REPO_ROOT").filter(|value| !value.is_empty());
    resolve_workspace_root(manifest_dir, runtime_root.as_deref())
}

fn resolve_workspace_root(manifest_dir: &str, runtime_root: Option<&OsStr>) -> PathBuf {
    if let Some(runtime_root) = runtime_root {
        return PathBuf::from(runtime_root);
    }
    Path::new(manifest_dir)
        .parent()
        .and_then(|path| path.parent())
        .unwrap_or_else(|| panic!("resolve workspace root from {manifest_dir}"))
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::resolve_workspace_root;
    use std::ffi::OsStr;
    use std::path::PathBuf;

    #[test]
    fn runtime_root_overrides_the_compile_time_manifest_checkout() {
        assert_eq!(
            resolve_workspace_root(
                "/archive-build/crates/bijux-dna-policies",
                Some(OsStr::new("/shard-workspace")),
            ),
            PathBuf::from("/shard-workspace"),
        );
    }

    #[test]
    fn manifest_layout_remains_the_local_fallback() {
        assert_eq!(
            resolve_workspace_root("/workspace/crates/bijux-dna-policies", None),
            PathBuf::from("/workspace"),
        );
    }
}
