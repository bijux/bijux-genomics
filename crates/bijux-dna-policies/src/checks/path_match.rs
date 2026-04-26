use std::path::Path;

pub(crate) fn path_has_allowed_suffix(path: &Path, allowed_suffixes: &[String]) -> bool {
    let normalized_path = normalize(path);
    allowed_suffixes.iter().any(|suffix| {
        let normalized_suffix = normalize(suffix);
        !normalized_suffix.is_empty() && normalized_path.ends_with(&normalized_suffix)
    })
}

fn normalize(path: impl AsRef<Path>) -> String {
    let parts = path
        .as_ref()
        .components()
        .filter_map(|component| {
            let text = component.as_os_str().to_string_lossy();
            (!text.is_empty() && text != "/").then(|| text.to_string())
        })
        .collect::<Vec<_>>();
    format!("/{}", parts.join("/"))
}
