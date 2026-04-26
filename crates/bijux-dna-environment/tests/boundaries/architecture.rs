use std::collections::BTreeSet;

#[test]
fn environment_tree_matches_architecture_contract() {
    let root = crate_root("bijux-dna-environment");

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
            "ENV_REFERENCE.md",
            "INDEX.md",
            "PUBLIC_API.md",
            "TESTS.md",
        ]),
        "docs must stay within the 10-document allowance"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        btree_set(&["build/", "lib.rs", "public_api/", "resolve/", "runtime_spec/",]),
        "src tree must match the documented environment layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/runtime_spec")),
        btree_set(&["compatibility.rs", "mod.rs", "model.rs"]),
        "runtime_spec tree must stay decomposed by runtime concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/build")),
        btree_set(&[
            "builder.rs",
            "defaults.rs",
            "entrypoints.rs",
            "mod.rs",
            "models.rs",
            "stable_surface.rs",
            "version_parser.rs",
        ]),
        "build tree must remain decomposed by concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolve")),
        btree_set(&[
            "cache/",
            "catalog/",
            "commands.rs",
            "entrypoints.rs",
            "facade.rs",
            "mod.rs",
            "platform.rs",
            "reference/",
            "shell.rs",
            "smoke.rs",
            "stable_surface.rs",
            "types/",
        ]),
        "resolve tree must remain decomposed by environment concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolve/cache")),
        btree_set(&["image_paths.rs", "mod.rs", "root.rs"]),
        "resolve cache tree must stay split by cache concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolve/catalog")),
        btree_set(
            &["catalog_loader.rs", "image_resolution.rs", "mod.rs", "registry_hydration.rs",]
        ),
        "resolve catalog tree must stay split by catalog concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolve/reference")),
        btree_set(&["digest.rs", "index_preparation.rs", "mod.rs"]),
        "resolve reference tree must stay split by reference concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolve/types")),
        btree_set(&["errors.rs", "image.rs", "mod.rs", "platform.rs", "runtime.rs"]),
        "resolve types tree must stay split by model concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        btree_set(&["mod.rs", "stable_surface.rs"]),
        "public api tree must keep the stable surface explicit"
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
            "schemas/",
            "schemas.rs",
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
