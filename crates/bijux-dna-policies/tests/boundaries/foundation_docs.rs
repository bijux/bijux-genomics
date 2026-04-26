use std::path::{Path, PathBuf};

const FOUNDATION_CRATES: &[&str] = &[
    "bijux-dna",
    "bijux-dna-api",
    "bijux-dna-core",
    "bijux-dna-dev",
    "bijux-dna-engine",
    "bijux-dna-infra",
    "bijux-dna-policies",
    "bijux-dna-runner",
    "bijux-dna-runtime",
    "bijux-dna-testkit",
];

#[test]
fn policy__boundaries__foundation_docs__foundation_crates_keep_docs_in_governed_locations() {
    let workspace = workspace_root();

    for crate_name in FOUNDATION_CRATES {
        let crate_root = workspace.join("crates").join(crate_name);
        let docs_dir = crate_root.join("docs");
        let docs_files = std::fs::read_dir(&docs_dir)
            .unwrap_or_else(|err| panic!("read {}: {err}", docs_dir.display()))
            .filter_map(|entry| {
                let path = entry.unwrap_or_else(|err| panic!("read docs entry: {err}")).path();
                (path.extension().and_then(|ext| ext.to_str()) == Some("md")).then_some(path)
            })
            .collect::<Vec<_>>();

        assert!(
            crate_root.join("README.md").is_file(),
            "{crate_name} must keep one root README.md"
        );
        assert_eq!(docs_files.len(), 10, "{crate_name} must keep exactly 10 docs files");

        for markdown in markdown_files(&crate_root) {
            let is_root_readme = markdown == crate_root.join("README.md");
            let is_docs_file = markdown.starts_with(&docs_dir);
            assert!(
                is_root_readme || is_docs_file,
                "{crate_name} has markdown outside root README.md and docs/: {}",
                markdown.display()
            );
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("resolve workspace root"))
        .to_path_buf()
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_markdown_files(root, &mut files);
    files
}

fn collect_markdown_files(current: &Path, files: &mut Vec<PathBuf>) {
    for entry in
        std::fs::read_dir(current).unwrap_or_else(|err| panic!("read {}: {err}", current.display()))
    {
        let entry =
            entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", current.display()));
        let path = entry.path();

        if path.is_dir() {
            collect_markdown_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.push(path);
        }
    }
}
