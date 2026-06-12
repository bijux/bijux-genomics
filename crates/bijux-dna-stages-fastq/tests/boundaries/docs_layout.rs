#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn markdown_docs_stay_in_root_readme_or_docs_dir() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let markdown_files = markdown_files(root);
    let expected = entries([
        "README.md",
        "docs/ARCHITECTURE.md",
        "docs/BOUNDARY.md",
        "docs/CHANGE_RULES.md",
        "docs/COMMANDS.md",
        "docs/DEPENDENCIES.md",
        "docs/EFFECTS.md",
        "docs/INDEX.md",
        "docs/PUBLIC_API.md",
        "docs/STAGE_CONTRACTS.md",
        "docs/TESTS.md",
    ]);

    assert_eq!(
        markdown_files, expected,
        "crate markdown must stay limited to root README.md and the 10 allowed docs/"
    );
}

#[test]
fn docs_index_references_every_allowed_doc() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let index = std::fs::read_to_string(root.join("docs/INDEX.md")).expect("read docs/INDEX.md");
    let expected = [
        "ARCHITECTURE.md",
        "BOUNDARY.md",
        "CHANGE_RULES.md",
        "COMMANDS.md",
        "DEPENDENCIES.md",
        "EFFECTS.md",
        "PUBLIC_API.md",
        "STAGE_CONTRACTS.md",
        "TESTS.md",
    ];

    for doc in expected {
        assert!(index.contains(doc), "docs/INDEX.md must reference docs/{doc}");
    }
}

fn markdown_files(root: &Path) -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    visit(root, root, &mut files);
    files
}

fn visit(root: &Path, path: &Path, files: &mut BTreeSet<String>) {
    for entry in
        std::fs::read_dir(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
        let path = entry.path();
        if path.is_dir() {
            visit(root, &path, files);
            continue;
        }
        if path.extension().is_some_and(|extension| extension == "md") {
            files.insert(relative_path(root, &path));
        }
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or_else(|err| panic!("strip {} from {}: {err}", root.display(), path.display()))
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
