use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn is_allowed_command_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/crates/bijux-runner/")
        || path_str.contains("/crates/bijux-environment/src/build/")
        || path_str.contains("/crates/bijux-environment/src/bin/")
        || path_str.contains("/crates/bijux-environment/src/resolve/")
}

#[test]
fn command_spawning_is_confined_to_runner_and_env_tooling() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = ["std::process::Command", "Command::new"];

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path.to_string_lossy().contains("/tests/") {
            continue;
        }
        if is_allowed_command_path(path) {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(path.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "std::process::Command must be confined to bijux-runner or bijux-environment tooling:\n{}",
        offenders.join("\n")
    );
}
