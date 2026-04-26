use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn db_ref_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
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

    assert_eq!(
        dir_entries(&root.join("docs")),
        entries([
            "ARCHITECTURE.md",
            "BOUNDARY.md",
            "CHANGE_RULES.md",
            "COMMANDS.md",
            "CONTRACTS.md",
            "DEPENDENCIES.md",
            "INDEX.md",
            "PUBLIC_API.md",
            "SCOPE.md",
            "TESTS.md",
        ]),
        "crate docs must stay under docs/ and within the 10-document allowance"
    );

    assert_eq!(
        dir_entries(&root.join("tests")),
        entries(["boundaries/", "boundaries.rs", "contracts/", "contracts.rs", "guardrails.rs"]),
        "integration tests must only track active test files and intent directories"
    );

    assert!(
        markdown_files_outside_docs(root).is_empty(),
        "crate markdown outside docs/ must be limited to root README.md"
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

fn markdown_files_outside_docs(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_markdown_files(root, root, &mut files);
    files
}

fn collect_markdown_files(root: &Path, path: &Path, files: &mut Vec<String>) {
    let entries =
        std::fs::read_dir(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(root, &path, files);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "md")
            && path != root.join("README.md")
            && !path.starts_with(root.join("docs"))
        {
            files.push(path.display().to_string());
        }
    }
}
