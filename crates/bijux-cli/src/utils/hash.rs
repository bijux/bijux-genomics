use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

pub fn hash_file_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read file: {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn normalize_run_base_dir(cwd: &Path, run_base: &Path) -> std::path::PathBuf {
    if run_base.is_absolute() {
        run_base.to_path_buf()
    } else {
        cwd.join(run_base)
    }
}

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
