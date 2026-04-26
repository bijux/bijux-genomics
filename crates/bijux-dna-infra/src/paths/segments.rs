use std::path::{Component, Path};

pub(super) fn path_segment(value: &str) -> String {
    let segments = confined_components(value);
    if segments.is_empty() {
        "_".to_string()
    } else {
        segments.join("_")
    }
}

fn confined_components(value: &str) -> Vec<String> {
    Path::new(value.trim())
        .components()
        .filter_map(|component| match component {
            Component::Normal(segment) => {
                let clean = segment
                    .to_string_lossy()
                    .chars()
                    .map(|ch| if matches!(ch, '/' | '\\' | '\0') { '_' } else { ch })
                    .collect::<String>();
                let clean = clean.trim();
                if clean.is_empty() { None } else { Some(clean.to_string()) }
            }
            Component::CurDir | Component::ParentDir | Component::RootDir => None,
            Component::Prefix(_) => None,
        })
        .collect()
}
