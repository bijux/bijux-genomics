#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

const MIN_MOD_LINES: usize = 5;

#[test]
fn no_empty_or_placeholder_dirs() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(&crate_root)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_dir())
        {
            let dir = entry.path();
            let mut rs_files = Vec::new();
            let mut has_any_file = false;
            if let Ok(read_dir) = std::fs::read_dir(dir) {
                for child in read_dir.flatten() {
                    let path = child.path();
                    if path.is_file() {
                        has_any_file = true;
                    }
                    if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                        rs_files.push(path);
                    }
                }
            }
            if !has_any_file {
                offenders.push(dir.display().to_string());
                continue;
            }
            if rs_files.len() == 1 && rs_files[0].file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
                let lines = support::read_to_string(&rs_files[0]).lines().count();
                if lines < MIN_MOD_LINES {
                    offenders.push(dir.display().to_string());
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "empty dirs or placeholder mod.rs modules are forbidden.\n\
Fix by removing empty dirs or adding real modules.\n\
See STYLE.md for tree rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
