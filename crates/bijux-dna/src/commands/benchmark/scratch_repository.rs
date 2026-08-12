use std::path::{Path, PathBuf};

fn link_entry(source: &Path, destination: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, destination).expect("link repository read root");

    #[cfg(windows)]
    if source.is_dir() {
        std::os::windows::fs::symlink_dir(source, destination)
            .expect("link repository read root");
    } else {
        std::os::windows::fs::symlink_file(source, destination)
            .expect("link repository read root");
    }
}

pub(crate) fn materialize_read_only_repository(test_name: &str) -> PathBuf {
    let source_root = crate::commands::support::workspace_root::resolve_repo_root()
        .expect("source repository root");
    let test_paths = bijux_dna_testkit::TestPaths::new(test_name);
    let scratch_root = test_paths.root().to_path_buf();

    for relative_root in [
        ".git",
        "assets",
        "benchmarks",
        "configs",
        "containers",
        "domain",
        "science",
    ] {
        let source = source_root.join(relative_root);
        if source.exists() {
            link_entry(&source, &scratch_root.join(relative_root));
        }
    }

    scratch_root
}
