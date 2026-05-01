use std::collections::BTreeSet;

#[test]
fn stages_vcf_tree_matches_architecture_contract() {
    let root = crate_root("bijux-dna-stages-vcf");

    assert_eq!(
        dir_entries(&root),
        btree_set(&["Cargo.toml", "README.md", "docs/", "examples/", "src/", "tests/",]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        btree_set(&[
            "engine/",
            "invariants.rs",
            "lib.rs",
            "metrics.rs",
            "path_contract.rs",
            "pipeline/",
            "pipeline_sections/",
            "repo_root.rs",
            "stage_specs.rs",
            "vcf_io.rs",
            "wrappers.rs",
        ]),
        "src tree must match the documented stages-vcf layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/engine")),
        btree_set(&[
            "entrypoints.rs",
            "mod.rs",
            "reporting.rs",
            "request.rs",
            "stage_runner.rs",
            "wrappers.rs",
        ]),
        "engine tree must remain decomposed by responsibility"
    );

    assert_eq!(
        dir_entries(&root.join("src/pipeline")),
        btree_set(&[
            "calling/",
            "imputation/",
            "mod.rs",
            "orchestration/",
            "population_panel/",
            "qc/",
        ]),
        "pipeline tree must remain decomposed by stage family"
    );
}

fn crate_root(crate_name: &str) -> std::path::PathBuf {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let actual = root.file_name().and_then(|name| name.to_str()).unwrap_or_default();
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
