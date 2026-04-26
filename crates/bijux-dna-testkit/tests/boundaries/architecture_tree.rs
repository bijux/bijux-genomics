use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn testkit_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "determinism/",
            "fixtures/",
            "lib.rs",
            "public_api/",
            "snapshots/",
            "temp/",
            "workspace_support/",
        ]),
        "src tree must match the documented test support layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/determinism")),
        entries([
            "OWNER.toml",
            "clock.rs",
            "json_assertions.rs",
            "mod.rs",
            "rng.rs",
            "timestamp_fields.rs",
        ]),
        "determinism tree must stay partitioned by support concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/fixtures")),
        entries(["OWNER.toml", "json_contracts.rs", "mod.rs", "readers.rs"]),
        "fixtures tree must keep readers and contracts separate"
    );

    assert_eq!(
        dir_entries(&root.join("src/snapshots")),
        entries([
            "OWNER.toml",
            "environment.rs",
            "json_normalization.rs",
            "mod.rs",
            "naming.rs",
            "text_normalization.rs",
        ]),
        "snapshots tree must keep naming, environment, and normalization concerns separate"
    );

    assert_eq!(
        dir_entries(&root.join("src/temp")),
        entries([
            "OWNER.toml",
            "directory_listing.rs",
            "mod.rs",
            "path_support.rs",
            "temp_dirs.rs",
            "test_paths.rs",
        ]),
        "temp tree must keep allocation, listing, and path models separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["OWNER.toml", "mod.rs", "surface.rs"]),
        "public api tree must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("src/workspace_support")),
        entries(["OWNER.toml", "mod.rs", "text.rs", "workspace_root.rs"]),
        "workspace support tree must keep root resolution and text loading explicit"
    );

    assert_eq!(
        dir_entries(&root.join("tests")),
        entries([
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism.rs",
            "guardrails.rs",
            "schemas/",
            "schemas.rs",
            "snapshots/",
        ]),
        "test tree must stay organized by enduring intent"
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
