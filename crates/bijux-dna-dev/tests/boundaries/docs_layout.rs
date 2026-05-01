use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const ALLOWED_DOCS: &[&str] = &[
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
];

#[test]
fn markdown_docs_stay_in_root_readme_or_docs_dir() {
    let root = crate_root();

    assert_eq!(
        markdown_files(&root),
        expected_markdown_files(&root),
        "bijux-dna-dev docs must stay at root README.md plus the governed docs directory files"
    );
}

#[test]
fn docs_index_references_command_and_dependency_contracts() {
    let root = crate_root();
    let index = std::fs::read_to_string(root.join("docs/INDEX.md"))
        .unwrap_or_else(|err| panic!("read docs/INDEX.md: {err}"));

    for doc in ["COMMANDS.md", "DEPENDENCIES.md", "PUBLIC_API.md", "TESTS.md"] {
        assert!(index.contains(doc), "docs/INDEX.md must reference docs/{doc}");
    }
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn markdown_files(root: &Path) -> BTreeSet<PathBuf> {
    let mut files = BTreeSet::new();
    collect_markdown_files(root, root, &mut files);
    files
}

fn collect_markdown_files(root: &Path, current: &Path, files: &mut BTreeSet<PathBuf>) {
    for entry in
        std::fs::read_dir(current).unwrap_or_else(|err| panic!("read {}: {err}", current.display()))
    {
        let entry =
            entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", current.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(root, &path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.insert(
                path.strip_prefix(root)
                    .unwrap_or_else(|err| panic!("strip {}: {err}", path.display()))
                    .to_path_buf(),
            );
        }
    }
}

fn expected_markdown_files(root: &Path) -> BTreeSet<PathBuf> {
    let mut expected = BTreeSet::from([PathBuf::from("README.md")]);
    for doc in ALLOWED_DOCS {
        expected.insert(PathBuf::from("docs").join(doc));
    }
    expected
        .insert(PathBuf::from("tests/snapshots/bijux-dna-dev__tooling__architecture_report.md"));
    for doc in &expected {
        assert!(root.join(doc).is_file(), "expected doc must exist: {}", doc.display());
    }
    expected
}
