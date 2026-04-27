use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn markdown_docs_stay_in_root_readme_or_docs_dir() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let markdown_files = markdown_files(root);
    let expected = entries([
        "README.md",
        "docs/ARCHITECTURE.md",
        "docs/BOUNDARY.md",
        "docs/COMMANDS.md",
        "docs/DEFAULTS_LEDGER.md",
        "docs/DEPENDENCIES.md",
        "docs/EFFECTS.md",
        "docs/INDEX.md",
        "docs/PIPELINES.md",
        "docs/PUBLIC_API.md",
        "docs/TESTS.md",
    ]);

    assert_eq!(
        markdown_files, expected,
        "crate documentation must keep one root README.md and exactly ten docs/*.md files"
    );
}

fn markdown_files(root: &Path) -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    visit(root, root, &mut files);
    files
}

fn visit(root: &Path, path: &Path, files: &mut BTreeSet<String>) {
    for entry in fs::read_dir(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display())) {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
        let path = entry.path();
        if path.is_dir() {
            visit(root, &path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            files.insert(relative_path(root, &path));
        }
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or_else(|err| panic!("strip root from {}: {err}", path.display()))
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
