use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .map_or_else(|| panic!("workspace root missing"), Path::to_path_buf)
}

fn is_allowed_command_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/crates/bijux-dna-runner/")
        || path_str.contains("/crates/bijux-dna-environment/src/build/")
        || path_str.contains("/crates/bijux-dna-environment/src/bin/")
        || path_str.contains("/crates/bijux-dna-environment/src/resolve/")
        || path_str.contains("/crates/bijux-dna-environment-qa/src/bin/")
        || path_str.contains("/crates/bijux-dna-environment-qa/src/image_qa/")
}

#[test]
fn command_spawning_is_confined_to_runner_and_env_tooling() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = [
        concat!("std::process::", "Command"),
        concat!("Command::", "new"),
        concat!("tokio::process::", "Command"),
    ];

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
        let content =
            std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read source: {err}"));
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(path.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "process command spawning must be confined to bijux-dna-runner or bijux-dna-environment tooling:\n{}",
        offenders.join("\n")
    );
}
