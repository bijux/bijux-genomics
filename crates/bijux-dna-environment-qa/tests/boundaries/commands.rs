use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use bijux_dna_testkit::sorted_read_dir_paths;

#[test]
fn command_inventory_matches_binary_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = std::fs::read_to_string(root.join("docs/COMMANDS.md"))
        .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));
    let bins = sorted_read_dir_paths(&root.join("src/bin"))
        .into_iter()
        .map(|path| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or_else(|| panic!("invalid binary name: {}", path.display()))
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let documented = documented_binary_names(&commands_doc);

    assert_eq!(documented, bins, "docs/COMMANDS.md must list every binary exactly");

    for command in [
        "docker build",
        "docker run",
        "docker image inspect",
        "docker images -q",
        "docker rm",
        "docker wait",
        "docker logs",
        "docker inspect",
        "gzip -t",
        "git rev-parse HEAD",
        "date -u +%Y-%m-%dT%H:%M:%SZ",
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

fn documented_binary_names(commands_doc: &str) -> BTreeSet<String> {
    let mut binaries = BTreeSet::new();
    let mut in_inventory = false;
    for line in commands_doc.lines() {
        if line.starts_with("## ") {
            in_inventory = line == "## Managed Command Inventory";
            continue;
        }
        if !in_inventory {
            continue;
        }
        if let Some(name) = line.trim().strip_prefix("- `").and_then(|value| value.split_once('`'))
        {
            binaries.insert(name.0.to_string());
        }
    }
    binaries
}

fn documented_process_files(commands_doc: &str) -> BTreeSet<PathBuf> {
    commands_doc
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- `src/"))
        .filter_map(|value| {
            value.split_once('`').map(|(path, _rest)| PathBuf::from("src").join(path))
        })
        .collect()
}

fn actual_process_files(root: &Path) -> BTreeSet<PathBuf> {
    let mut files = BTreeSet::new();
    collect_process_files(root, &root.join("src"), &mut files);
    files
}

fn collect_process_files(root: &Path, dir: &Path, files: &mut BTreeSet<PathBuf>) {
    for entry in sorted_read_dir_paths(dir) {
        if entry.is_dir() {
            collect_process_files(root, &entry, files);
            continue;
        }
        if entry.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(&entry)
            .unwrap_or_else(|err| panic!("read {}: {err}", entry.display()));
        if content.contains("std::process::Command") || content.contains("Command::new") {
            files.insert(
                entry
                    .strip_prefix(root)
                    .unwrap_or_else(|err| panic!("strip crate root: {err}"))
                    .to_path_buf(),
            );
        }
    }
}
