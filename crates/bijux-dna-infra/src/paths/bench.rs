use std::path::{Path, PathBuf};

use super::segments::path_segment;

#[must_use]
pub fn bench_base_dir(out: &Path, stage: &str, sample_id: &str) -> PathBuf {
    out.join("bench").join(path_segment(stage)).join(path_segment(sample_id))
}

#[must_use]
pub fn bench_tools_dir(out: &Path, stage: &str, sample_id: &str) -> PathBuf {
    bench_base_dir(out, stage, sample_id).join("tools")
}

#[must_use]
pub fn bench_data_dir(root: &Path) -> PathBuf {
    root.join("crates").join("bijux-dna-bench").join("bench")
}

#[must_use]
pub fn bench_suites_dir(root: &Path) -> PathBuf {
    bench_data_dir(root).join("suites")
}
