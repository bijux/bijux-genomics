use std::collections::BTreeSet;

#[test]
fn stages_bam_tree_matches_architecture_contract() {
    let root = crate_root("bijux-dna-stages-bam");

    assert_eq!(
        dir_entries(&root),
        btree_set(&[
            "BOUNDARY.md",
            "Cargo.toml",
            "PUBLIC_API.md",
            "README.md",
            "docs/",
            "src/",
            "tests/",
        ]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        btree_set(&[
            "lib.rs",
            "metrics/",
            "observer.rs",
            "plugin/",
            "stage_specs.rs",
            "surface.rs",
        ]),
        "src tree must match the documented stages-bam layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/metrics")),
        btree_set(&[
            "alignment.rs",
            "contamination.rs",
            "coverage.rs",
            "damage.rs",
            "discovery.rs",
            "mod.rs",
            "quality.rs",
        ]),
        "metrics tree must remain decomposed by BAM metric concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/plugin")),
        btree_set(&["invocation.rs", "mod.rs", "output/"]),
        "plugin tree must remain decomposed by plugin responsibility"
    );
}

fn crate_root(crate_name: &str) -> std::path::PathBuf {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let actual = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    assert_eq!(actual, crate_name, "unexpected integration-test crate root");
    root
}

fn dir_entries(path: &std::path::Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect()
}

fn btree_set(entries: &[&str]) -> BTreeSet<String> {
    entries.iter().map(|entry| (*entry).to_string()).collect()
}
