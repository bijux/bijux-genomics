use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn assert_no_imports(dir: &str, forbidden: &[&str]) {
    let mut files = Vec::new();
    collect_rs_files(Path::new(dir), &mut files);
    for file in files {
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        for needle in forbidden {
            assert!(
                !contents.contains(needle),
                "forbidden import in {}: {}",
                file.display(),
                needle
            );
        }
    }
}

#[test]
fn cli_does_not_import_engine_internals() {
    assert_no_imports(
        "crates/bijux-cli/src",
        &[
            "bijux_engine::internal",
            "bijux_engine::planner::",
            "bijux_engine::executor::",
            "bijux_engine::observer::",
            "bijux_engine::validator::",
            "bijux_engine::types::",
        ],
    );
}
