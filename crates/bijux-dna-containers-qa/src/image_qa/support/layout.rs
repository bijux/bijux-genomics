use std::path::{Path, PathBuf};

#[must_use]
pub fn image_qa_base_dir(cwd: &Path, platform: &str) -> PathBuf {
    cwd.join("artifacts").join("image-qa").join(platform)
}

#[must_use]
pub fn image_qa_jsonl_path(cwd: &Path, platform: &str) -> PathBuf {
    image_qa_base_dir(cwd, platform).join("qa.jsonl")
}

#[must_use]
pub fn image_qa_sqlite_path(cwd: &Path, platform: &str) -> PathBuf {
    image_qa_base_dir(cwd, platform).join("qa.sqlite")
}
