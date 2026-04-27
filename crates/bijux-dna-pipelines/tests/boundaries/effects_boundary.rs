use std::fs;
use std::path::Path;

#[test]
fn source_has_no_process_or_network_effects() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let source = rust_source(&root);

    for forbidden in [
        "std::process",
        "Command::new",
        ".spawn(",
        "TcpStream",
        "TcpListener",
        "UdpSocket",
        "reqwest::",
    ] {
        assert!(
            !source.contains(forbidden),
            "bijux-dna-pipelines source must not use forbidden effect primitive `{forbidden}`"
        );
    }
}

fn rust_source(path: &Path) -> String {
    let mut source = String::new();
    collect_rust_source(path, &mut source);
    source
}

fn collect_rust_source(path: &Path, source: &mut String) {
    for entry in fs::read_dir(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display())) {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source(&path, source);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            source.push_str(
                &fs::read_to_string(&path)
                    .unwrap_or_else(|err| panic!("read {}: {err}", path.display())),
            );
            source.push('\n');
        }
    }
}
