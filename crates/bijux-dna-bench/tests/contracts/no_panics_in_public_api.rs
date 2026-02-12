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
fn no_panics_in_public_api() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    let mut offenders = Vec::new();
    for file in files {
        let path_str = file.to_string_lossy();
        if path_str.contains("/tests/") {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        for needle in ["panic!(", ".unwrap()", ".expect("] {
            if contents.contains(needle) {
                offenders.push(format!("{path_str}: {needle}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "panic/unwrap/expect found in src: {offenders:?}"
    );
}
