use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const FORBIDDEN_EFFECT_TOKENS: &[&str] = &[
    "std::process",
    "Command::new",
    "std::net",
    "TcpStream",
    "UdpSocket",
    "reqwest::",
    "ureq::",
    "hyper::",
    "std::fs::write",
    "std::fs::remove_file",
    "std::fs::rename",
    "File::create",
];

#[test]
fn production_source_rejects_process_network_and_source_mutation_effects() {
    for file in rust_source_files(&crate_root().join("src")) {
        let source = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("read {}: {err}", file.display()));

        for forbidden in FORBIDDEN_EFFECT_TOKENS {
            assert!(
                !source.contains(forbidden),
                "testkit production source must not contain effect token {forbidden} in {}",
                file.display()
            );
        }
    }
}

#[test]
fn production_environment_reads_are_documented() {
    let root = crate_root();
    let source_vars = env_vars_in_source(&root.join("src"));
    let effects = std::fs::read_to_string(root.join("docs/EFFECTS.md"))
        .unwrap_or_else(|err| panic!("read docs/EFFECTS.md: {err}"));

    for var in &source_vars {
        assert!(effects.contains(var), "docs/EFFECTS.md must document environment variable {var}");
    }
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

fn env_vars_in_source(root: &Path) -> BTreeSet<String> {
    let mut vars = BTreeSet::new();

    for file in rust_source_files(root) {
        let source = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("read {}: {err}", file.display()));
        for (index, token) in source.split('"').enumerate() {
            if index % 2 == 1
                && token.chars().all(|ch| ch.is_ascii_uppercase() || ch == '_')
                && token.contains('_')
                && !token.starts_with('_')
            {
                vars.insert(token.to_string());
            }
        }
    }

    vars
}
