use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn science_source_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "science crate root must stay minimal"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "app/",
            "cli.rs",
            "compile.rs",
            "domain/",
            "errors.rs",
            "io.rs",
            "lib.rs",
            "main.rs",
            "release.rs",
            "render/",
            "schema/",
        ]),
        "science src tree must stay grouped by command, compile, domain, IO, release, render, and schema concerns"
    );

    assert_eq!(
        dir_entries(&root.join("src/app")),
        entries(["mod.rs"]),
        "science app command coordinator must stay isolated"
    );
    assert_eq!(
        dir_entries(&root.join("src/domain")),
        entries(["mod.rs"]),
        "science domain types must stay isolated"
    );
    assert_eq!(
        dir_entries(&root.join("src/render")),
        entries(["mod.rs"]),
        "science render functions must stay isolated"
    );
    assert_eq!(
        dir_entries(&root.join("src/schema")),
        entries(["mod.rs"]),
        "science schema constants must stay isolated"
    );
}

#[test]
fn science_test_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(&root.join("tests")),
        entries([
            "boundaries/",
            "boundaries.rs",
            "contracts.rs",
            "fastq_environment_slice.rs",
            "guardrails.rs",
            "tsv_shape.rs",
        ]),
        "science tests must stay grouped by boundary, contract, generated-output, guardrail, and TSV shape concerns"
    );

    assert_eq!(
        dir_entries(&root.join("tests/boundaries")),
        entries([
            "architecture.rs",
            "command_inventory.rs",
            "dependency_graph.rs",
            "docs_layout.rs",
            "generated_surface_docs.rs",
            "public_api_docs.rs",
        ]),
        "science boundary tests must cover docs, commands, generated surfaces, dependencies, public API, and architecture"
    );
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
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

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
