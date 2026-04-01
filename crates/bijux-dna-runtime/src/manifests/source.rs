use std::path::{Path, PathBuf};

#[must_use]
pub fn find_domain_dir(path: &Path) -> Option<PathBuf> {
    if path.is_dir()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "domain")
    {
        return Some(path.to_path_buf());
    }
    if path.file_name().and_then(|name| name.to_str()) == Some("tool_registry.toml") {
        let parent = path.parent()?;
        if parent.file_name().and_then(|name| name.to_str()) == Some("configs") {
            let candidate = parent.parent()?.join("domain");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    if path.is_dir() {
        let candidate = path.join("domain");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[must_use]
pub fn experimental_manifests_enabled() -> bool {
    [
        "BIJUX_INCLUDE_EXPERIMENTAL_TOOLS",
        "BIJUX_EXPERIMENTAL_TOOLS",
    ]
    .into_iter()
    .filter_map(|key| std::env::var(key).ok())
    .any(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}
