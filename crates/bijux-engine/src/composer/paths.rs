use std::path::Path;

pub fn bench_base_dir(out: &Path, stage: &str, sample_id: &str) -> std::path::PathBuf {
    out.join("artifacts")
        .join("bench")
        .join(stage)
        .join(sample_id)
}

pub fn bench_tools_dir(out: &Path, stage: &str, sample_id: &str) -> std::path::PathBuf {
    out.join("artifacts")
        .join("bench")
        .join(stage)
        .join(sample_id)
        .join("tools")
}

pub fn image_qa_base_dir(cwd: &Path, platform: &str) -> std::path::PathBuf {
    cwd.join("artifacts").join("image-qa").join(platform)
}

pub fn image_qa_jsonl_path(cwd: &Path, platform: &str) -> std::path::PathBuf {
    image_qa_base_dir(cwd, platform).join("qa.jsonl")
}

pub fn image_qa_sqlite_path(cwd: &Path, platform: &str) -> std::path::PathBuf {
    image_qa_base_dir(cwd, platform).join("qa.sqlite")
}
