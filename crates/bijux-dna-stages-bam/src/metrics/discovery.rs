use std::path::{Path, PathBuf};

pub(super) fn first_existing(out_dir: &Path, names: &[&str]) -> Option<PathBuf> {
    for name in names {
        let candidate = out_dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
