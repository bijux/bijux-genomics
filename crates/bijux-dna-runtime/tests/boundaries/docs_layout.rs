use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn runtime_docs_stay_in_root_readme_and_ten_docs_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        markdown_files(root),
        entries([
            "README.md",
            "docs/ARCHITECTURE.md",
            "docs/ARTIFACTS.md",
            "docs/BOUNDARY.md",
            "docs/COMMANDS.md",
            "docs/DEPENDENCIES.md",
            "docs/EFFECTS.md",
            "docs/INDEX.md",
            "docs/PUBLIC_API.md",
            "docs/RUNTIME_CONTRACT.md",
            "docs/TESTS.md",
        ]),
        "runtime docs must be one root README plus exactly ten docs/ files"
    );

    let docs_entries = fs::read_dir(root.join("docs"))
        .unwrap_or_else(|err| panic!("read runtime docs dir: {err}"))
        .count();
    assert_eq!(docs_entries, 10, "runtime docs allowance is ten files");
}

fn markdown_files(root: &Path) -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    collect_markdown(root, root, &mut files);
    files
}

fn collect_markdown(root: &Path, current: &Path, files: &mut BTreeSet<String>) {
    for entry in
        fs::read_dir(current).unwrap_or_else(|err| panic!("read {}: {err}", current.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", current.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(root, &path, files);
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
        .replace('\\', "/")
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
