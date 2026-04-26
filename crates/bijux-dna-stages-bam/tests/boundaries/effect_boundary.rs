use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[test]
fn production_code_has_no_forbidden_effects() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let forbidden = [
        "std::process",
        "Command::",
        "tokio::process",
        "std::fs::write",
        "write_bytes",
        "create_dir",
        "remove_file",
        "remove_dir",
        "TcpStream",
        "UdpSocket",
        "reqwest",
    ];
    let mut offenders = Vec::new();

    for path in rust_source_files(&root.join("src")) {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        for needle in forbidden {
            if content.contains(needle) {
                offenders.push(format!("{} contains `{needle}`", path.display()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "stages-bam production code must stay parsing/materialization only:\n{}",
        offenders.join("\n")
    );
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("rs"))
        .collect()
}
