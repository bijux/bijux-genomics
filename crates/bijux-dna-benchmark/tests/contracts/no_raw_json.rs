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

#[test]
fn serde_json_value_is_confined_to_repo_and_artifacts() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    let mut offenders = Vec::new();
    for file in files {
        let path_str = file.to_string_lossy();
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        if contents.contains("serde_json::Value")
            && !path_str.contains("/repo/")
            && !path_str.contains("/artifacts/")
        {
            offenders.push(path_str.to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "serde_json::Value outside repo/artifacts: {offenders:?}"
    );
}
