use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn db_ref_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries([
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
        entries([
            "catalog/",
            "lib.rs",
            "model/",
            "providers/",
            "public_api/",
            "resolution/",
            "runtime_config/",
        ]),
        "src tree must match the documented reference resolver layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/catalog")),
        entries(["compatibility.rs", "entries.rs", "locks.rs", "mod.rs"]),
        "catalog tree must separate entries, locks, and compatibility policy"
    );

    assert_eq!(
        dir_entries(&root.join("src/model")),
        entries(["mod.rs", "reference_assets.rs", "species.rs"]),
        "model tree must separate species and reference asset contracts"
    );

    assert_eq!(
        dir_entries(&root.join("src/providers")),
        entries(["contracts.rs", "mod.rs", "runtime.rs"]),
        "providers tree must separate contracts from runtime-backed resolution"
    );

    assert_eq!(
        dir_entries(&root.join("src/resolution")),
        entries([
            "compatibility.rs",
            "locks.rs",
            "maps.rs",
            "mod.rs",
            "panels.rs",
            "reference_assets.rs",
            "species.rs",
        ]),
        "resolution tree must stay focused on lookup behavior"
    );

    assert_eq!(
        dir_entries(&root.join("src/runtime_config")),
        entries([
            "authority.rs",
            "bundles.rs",
            "catalogs.rs",
            "load.rs",
            "mod.rs",
            "paths.rs",
            "references.rs",
        ]),
        "runtime config tree must group TOML schemas by concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["mod.rs"]),
        "public api tree must stay curated"
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
