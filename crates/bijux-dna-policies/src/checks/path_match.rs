use std::path::Path;

pub(crate) fn path_has_allowed_suffix(path: &Path, allowed_suffixes: &[String]) -> bool {
    let normalized_path = normalize(path);
    allowed_suffixes.iter().any(|suffix| {
        let normalized_suffix = normalize(suffix);
        !normalized_suffix.is_empty() && normalized_path.ends_with(&normalized_suffix)
    })
}

fn normalize(path: impl AsRef<Path>) -> String {
    path.as_ref().components().fold(String::new(), |mut out, component| {
        if !out.ends_with('/') {
            out.push('/');
        }
        out.push_str(&component.as_os_str().to_string_lossy());
        out
    })
}
