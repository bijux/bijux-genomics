use std::path::{Path, PathBuf};

#[test]
fn source_tree_does_not_gain_execution_network_or_mutation_effects() {
    let source_files = rust_files_under(Path::new(env!("CARGO_MANIFEST_DIR")).join("src"));
    let forbidden = [
        "Command::new",
        ".spawn(",
        "std::process",
        "tokio::process",
        "std::net::",
        "TcpStream",
        "UdpSocket",
        "reqwest::",
        "ureq::",
        "hyper::",
        "tokio::net",
        "std::fs::write",
        "fs::write",
        "File::create",
        "create_dir",
        "remove_file",
        "remove_dir",
    ];

    for path in source_files {
        let source =
            std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {path:?}: {err}"));
        for needle in forbidden {
            assert!(
                !source.contains(needle),
                "planner source must not contain effect API `{needle}` in {path:?}"
            );
        }
    }
}

fn rust_files_under(root: PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(&root, &mut files);
    files.sort();
    files
}

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|err| panic!("read dir {dir:?}: {err}"));
    for entry in entries {
        let path = entry.unwrap_or_else(|err| panic!("read dir entry in {dir:?}: {err}")).path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
