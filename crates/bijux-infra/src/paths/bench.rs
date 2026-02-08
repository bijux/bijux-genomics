use std::path::{Path, PathBuf};

#[must_use]
pub fn bench_base_dir(out: &Path, stage: &str, sample_id: &str) -> PathBuf {
    out.join("bench").join(stage).join(sample_id)
}

#[must_use]
pub fn bench_tools_dir(out: &Path, stage: &str, sample_id: &str) -> PathBuf {
    bench_base_dir(out, stage, sample_id).join("tools")
}
