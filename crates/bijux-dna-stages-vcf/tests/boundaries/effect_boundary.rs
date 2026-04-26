use std::path::{Path, PathBuf};

const FORBIDDEN_NETWORK_TOKENS: &[&str] = &[
    "std::net::",
    "TcpStream",
    "UdpSocket",
    "reqwest::",
    "ureq::",
    "hyper::",
    "Command::new(\"curl\")",
    "Command::new(\"wget\")",
    "std::process::Command::new(\"curl\")",
    "std::process::Command::new(\"wget\")",
];

#[test]
fn production_source_rejects_network_effect_apis() {
    for file in rust_source_files(&crate_root().join("src")) {
        let source = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("read {}: {err}", file.display()));

        for forbidden in FORBIDDEN_NETWORK_TOKENS {
            assert!(
                !source.contains(forbidden),
                "stages-vcf must not introduce network effect token {forbidden} in {}",
                file.display()
            );
        }
    }
}

#[test]
fn allow_network_override_remains_refusal_only() {
    let stage_runner = std::fs::read_to_string(crate_root().join("src/engine/stage_runner.rs"))
        .unwrap_or_else(|err| panic!("read stage_runner.rs: {err}"));

    assert!(
        stage_runner.contains("BIJUX_VCF_ALLOW_NETWORK")
            && stage_runner.contains("no-network policy violation")
            && stage_runner.contains("is not permitted"),
        "BIJUX_VCF_ALLOW_NETWORK must remain a refusal guard, not an enablement switch"
    );
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(current: &Path, files: &mut Vec<PathBuf>) {
    for entry in
        std::fs::read_dir(current).unwrap_or_else(|err| panic!("read {}: {err}", current.display()))
    {
        let entry =
            entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", current.display()));
        let path = entry.path();

        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
