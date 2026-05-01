use std::path::{Path, PathBuf};

pub fn repo_root() -> PathBuf {
    let Some(root) = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent) else {
        panic!("repo root");
    };
    root.to_path_buf()
}

#[allow(dead_code)]
pub fn artifact_output_dir(prefix: &str) -> anyhow::Result<tempfile::TempDir> {
    let root = repo_root();
    let run_root = root.join("artifacts/domain-compiler-test-runs");
    std::fs::create_dir_all(&run_root)?;
    tempfile::Builder::new().prefix(prefix).tempdir_in(run_root).map_err(Into::into)
}
