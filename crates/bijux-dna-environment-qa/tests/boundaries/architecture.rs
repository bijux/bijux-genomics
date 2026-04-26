use std::collections::BTreeSet;

#[test]
fn environment_qa_tree_matches_architecture_contract() {
    let root = crate_root("bijux-dna-environment-qa");

    assert_eq!(
        dir_entries(&root),
        btree_set(&["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("docs")),
        btree_set(&[
            "ARCHITECTURE.md",
            "BOUNDARY.md",
            "COMMANDS.md",
            "CONTRACTS.md",
            "DEPENDENCIES.md",
            "EFFECTS.md",
            "IMAGE_QA.md",
            "INDEX.md",
            "PUBLIC_API.md",
            "TESTS.md",
        ]),
        "docs must stay within the 10-document allowance"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        btree_set(&["bin/", "image_qa/", "lib.rs", "public_api.rs"]),
        "src tree must match the documented environment qa layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/image_qa")),
        btree_set(&[
            "apptainer.rs",
            "behavioral/",
            "contracts.rs",
            "datasets/",
            "facade.rs",
            "fs.rs",
            "logging.rs",
            "mod.rs",
            "qa_docker_images/",
            "records/",
            "runner.rs",
            "static_qa.rs",
            "support/",
            "validation/",
        ]),
        "image_qa tree must remain decomposed by concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/image_qa/behavioral")),
        btree_set(&["mod.rs", "postprocess.rs", "preprocess.rs", "scenarios.rs"]),
        "behavioral tree must stay split by stage concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/image_qa/qa_docker_images")),
        btree_set(&[
            "args.rs",
            "contracts.rs",
            "mod.rs",
            "models.rs",
            "planning.rs",
            "probe.rs",
            "reporting.rs",
            "runtime.rs",
        ]),
        "docker qa tree must stay split by execution concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/image_qa/records")),
        btree_set(&["builder.rs", "mod.rs", "pass_cache.rs", "store.rs"]),
        "record tree must stay split by persistence concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/image_qa/support")),
        btree_set(&[
            "diagnostics.rs",
            "docker_exec/",
            "docker_runtime.rs",
            "execution_models.rs",
            "image_resolution.rs",
            "layout.rs",
            "mod.rs",
            "output_contracts.rs",
            "seqkit.rs",
        ]),
        "support tree must stay split by helper concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/image_qa/support/docker_exec")),
        btree_set(&["inspection.rs", "merge.rs", "mod.rs", "models.rs", "transform.rs"]),
        "docker_exec tree must keep command builders and models separated"
    );

    assert_eq!(
        dir_entries(&root.join("tests")),
        btree_set(&[
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "determinism.rs",
            "fixtures/",
            "guardrails.rs",
            "support/",
        ]),
        "test tree must stay grouped by taxonomy without nested README docs"
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
