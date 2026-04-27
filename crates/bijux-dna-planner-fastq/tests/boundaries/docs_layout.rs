use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn markdown_docs_stay_in_root_readme_or_docs_dir() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_eq!(
        markdown_files(root),
        entries([
            "README.md",
            "docs/ARCHITECTURE.md",
            "docs/BOUNDARY.md",
            "docs/COMMANDS.md",
            "docs/DEPENDENCIES.md",
            "docs/DETERMINISM.md",
            "docs/EFFECTS.md",
            "docs/EXPLAIN_OUTPUT.md",
            "docs/INDEX.md",
            "docs/PUBLIC_API.md",
            "docs/TESTS.md",
        ]),
        "crate docs must stay as root README.md plus exactly ten docs under docs/"
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
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            let relative =
                path.strip_prefix(root).unwrap_or_else(|_| panic!("path under crate root"));
            files.insert(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
