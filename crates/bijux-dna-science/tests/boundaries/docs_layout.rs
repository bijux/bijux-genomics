use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn science_docs_stay_in_root_readme_and_ten_docs_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut markdown = BTreeSet::new();
    collect_markdown(root, root, &mut markdown);

    assert_eq!(
        markdown,
        entries([
            "README.md",
            "docs/ARCHITECTURE.md",
            "docs/BOUNDARY.md",
            "docs/COMMANDS.md",
            "docs/CONTRACT.md",
            "docs/DEPENDENCIES.md",
            "docs/INDEX.md",
            "docs/PUBLIC_API.md",
            "docs/SCHEMAS.md",
            "docs/TESTS.md",
            "docs/VERSIONING.md",
        ]),
        "bijux-dna-science must keep one root README and exactly ten docs files"
    );
}

fn collect_markdown(root: &Path, current: &Path, files: &mut BTreeSet<String>) {
    for entry in
        fs::read_dir(current).unwrap_or_else(|err| panic!("read {}: {err}", current.display()))
    {
        let entry =
            entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", current.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(root, &path, files);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.insert(relative(root, &path));
        }
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
