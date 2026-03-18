use std::fs;
use std::path::Path;

fn collect_rs_files(root: &Path, files: &mut Vec<std::path::PathBuf>) {
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
fn cli_does_not_spawn_processes() {
    let mut files = Vec::new();
    collect_rs_files(Path::new("../../../src"), &mut files);
    let mut offenders = Vec::new();
    let needles = [
        concat!("std::process::", "Command"),
        concat!("Command::", "new("),
        concat!("tokio::process::", "Command"),
    ];
    for file in files {
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        for needle in &needles {
            if contents.contains(needle) {
                offenders.push(format!("{} contains {}", file.display(), needle));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "cli must not spawn processes directly:\n{}",
        offenders.join("\n")
    );
}
