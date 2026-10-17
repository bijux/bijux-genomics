#![cfg(test)]
use std::path::{Path, PathBuf};
use std::process::Command;

fn link_entry(source: &Path, destination: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, destination).expect("link repository read root");

    #[cfg(windows)]
    if source.is_dir() {
        std::os::windows::fs::symlink_dir(source, destination).expect("link repository read root");
    } else {
        std::os::windows::fs::symlink_file(source, destination).expect("link repository read root");
    }
}

pub(crate) fn materialize_read_only_repository(test_name: &str) -> PathBuf {
    let source_root = crate::commands::support::workspace_root::resolve_repo_root()
        .expect("source repository root");
    let test_paths = bijux_dna_testkit::TestPaths::new(test_name);
    let scratch_root = test_paths.root().to_path_buf();

    for relative_root in
        [".git", "assets", "benchmarks", "configs", "containers", "domain", "science"]
    {
        let source = source_root.join(relative_root);
        if source.exists() {
            link_entry(&source, &scratch_root.join(relative_root));
        }
    }

    scratch_root
}

pub(crate) fn materialize_repository_sandbox(test_name: &str) -> tempfile::TempDir {
    let source_root = crate::commands::support::workspace_root::resolve_repo_root()
        .expect("source repository root");
    let parent = source_root.join("artifacts/test-sandboxes");
    std::fs::create_dir_all(&parent).expect("create repository sandbox parent");
    let sandbox = tempfile::Builder::new()
        .prefix(test_name)
        .tempdir_in(parent)
        .expect("create repository sandbox");
    let output = Command::new("git")
        .current_dir(&source_root)
        .args(["checkout-index", "--all", "--force"])
        .arg(format!("--prefix={}/", sandbox.path().display()))
        .output()
        .expect("materialize tracked repository inputs");
    assert!(
        output.status.success(),
        "materialize repository sandbox: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Repository identity is read-only; mutable fixture and report paths are
    // copied so independent tests never publish into another test's source.
    link_entry(&source_root.join(".git"), &sandbox.path().join(".git"));
    sandbox
}
