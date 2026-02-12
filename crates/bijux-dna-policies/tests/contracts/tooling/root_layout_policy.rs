#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn policy__contracts__root_layout_policy__top_level_directories_are_allowlisted() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let allowed: BTreeSet<&str> = [
        "crates",
        "configs",
        "containers",
        "docs",
        "examples",
        "bench",
        "scripts",
        "artifacts",
        "bin",
        "domain",
        "assets",
        "makefiles",
    ]
    .into_iter()
    .collect();

    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(root).expect("read workspace root") {
        let entry = entry.expect("read root entry");
        if !entry.file_type().expect("file type").is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        if !allowed.contains(name.as_str()) {
            offenders.push(name);
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "root contains non-allowlisted directories: {:?}",
        offenders
    );
}
