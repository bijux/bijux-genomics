use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[test]
fn command_inventory_matches_process_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = std::fs::read_to_string(root.join("docs/COMMANDS.md"))
        .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));

    assert!(
        commands_doc.contains("library crate, not a CLI package"),
        "COMMANDS.md must declare that the crate has no CLI package surface"
    );
    assert!(
        commands_doc.contains("## Managed Command Inventory"),
        "COMMANDS.md must include the managed command inventory section"
    );
    assert!(!root.join("src/bin").exists(), "environment must not grow binary entrypoints");
    assert!(!root.join("src/main.rs").exists(), "environment must not grow a main binary");
    assert!(!manifest.contains("[[bin]]"), "environment must not declare Cargo binaries");

    for command in [
        "docker --version",
        "apptainer --version",
        "singularity --version",
        "docker image inspect",
        "sh -lc",
        "cargo run -q -p bijux-dna-dev",
        "samtools faidx",
        "gatk",
        "bwa index",
        "bowtie2-build",
    ] {
        assert!(commands_doc.contains(command), "COMMANDS.md missing `{command}`");
    }
}

#[test]
fn process_spawning_files_are_documented() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = std::fs::read_to_string(root.join("docs/COMMANDS.md"))
        .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));
    let documented = documented_process_files(&commands_doc);
    let actual = actual_process_files(root);

    assert_eq!(
        actual, documented,
        "std::process::Command usage must stay documented in docs/COMMANDS.md"
    );
}

fn documented_process_files(commands_doc: &str) -> BTreeSet<PathBuf> {
    let mut files = BTreeSet::new();
    for line in commands_doc.lines() {
        if let Some(path) = line.trim().strip_prefix("- `src/").and_then(|value| {
            value.split_once('`').map(|(path, _rest)| PathBuf::from("src").join(path))
        }) {
            files.insert(path);
        }
    }
    files
}

fn actual_process_files(root: &Path) -> BTreeSet<PathBuf> {
    let mut files = BTreeSet::new();
    for entry in WalkDir::new(root.join("src")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|err| panic!("read {}: {err}", entry.path().display()));
        if content.contains("std::process::Command") || content.contains("Command::new") {
            files.insert(
                entry
                    .path()
                    .strip_prefix(root)
                    .unwrap_or_else(|err| panic!("strip crate root: {err}"))
                    .to_path_buf(),
            );
        }
    }
    files
}
