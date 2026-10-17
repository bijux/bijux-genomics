use std::path::{Path, PathBuf};

pub(super) enum MetricInputs<'a> {
    Directory(&'a Path),
    Declared(&'a [PathBuf]),
}

pub(super) fn first_existing(inputs: &MetricInputs<'_>, names: &[&str]) -> Option<PathBuf> {
    for name in names {
        let candidate = match inputs {
            MetricInputs::Directory(out_dir) => {
                let candidate = out_dir.join(name);
                candidate.is_file().then_some(candidate)
            }
            MetricInputs::Declared(paths) => paths
                .iter()
                .filter(|path| path.file_name() == Some(std::ffi::OsStr::new(name)))
                .filter(|path| path.is_file())
                .min()
                .cloned(),
        };
        if candidate.is_some() {
            return candidate;
        }
    }
    None
}
