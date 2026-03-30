use std::path::{Path, PathBuf};

pub(super) fn common_parent(paths: &[PathBuf]) -> Option<PathBuf> {
    let mut iter = paths.iter();
    let first = iter.next()?.clone();
    let mut prefix = first;
    for path in iter {
        while !path.starts_with(&prefix) {
            if !prefix.pop() {
                return None;
            }
        }
    }
    Some(prefix)
}

pub(super) fn container_input_mapping(input_root: &Path) -> (PathBuf, String) {
    if input_root.is_file() {
        let mount_root = input_root
            .parent()
            .map_or_else(|| PathBuf::from("/"), Path::to_path_buf);
        let file_name = input_root.file_name().map_or_else(
            || "input".to_string(),
            |name| name.to_string_lossy().into_owned(),
        );
        return (mount_root, format!("/data/input/{file_name}"));
    }
    (input_root.to_path_buf(), "/data/input".to_string())
}

pub(super) fn preserve_absolute_input_paths(inputs: &[PathBuf]) -> bool {
    common_parent(inputs).is_some_and(|path| path == Path::new("/"))
}

fn input_bind_root(input: &Path) -> PathBuf {
    if input.exists() {
        if input.is_file() {
            return input
                .parent()
                .map_or_else(|| PathBuf::from("/"), Path::to_path_buf);
        }
        return input.to_path_buf();
    }
    input
        .parent()
        .map_or_else(|| PathBuf::from("/"), Path::to_path_buf)
}

fn collapse_bind_roots(mut roots: Vec<PathBuf>) -> Vec<PathBuf> {
    roots.sort_by(|left, right| {
        left.components()
            .count()
            .cmp(&right.components().count())
            .then_with(|| left.cmp(right))
    });
    let mut collapsed = Vec::<PathBuf>::new();
    for root in roots {
        if collapsed.iter().any(|existing| root.starts_with(existing)) {
            continue;
        }
        collapsed.push(root);
    }
    collapsed
}

pub(super) fn input_bind_roots(
    inputs: &[PathBuf],
    input_root: &Path,
    preserve_absolute: bool,
) -> Vec<PathBuf> {
    if !preserve_absolute {
        return vec![container_input_mapping(input_root).0];
    }
    collapse_bind_roots(inputs.iter().map(|input| input_bind_root(input)).collect())
}
