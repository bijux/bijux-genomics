use std::path::{Path, PathBuf};

#[test]
fn crate_markdown_layout_is_bounded() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let markdown_files = collect_markdown(root);
    let root_readme = root.join("README.md");
    assert!(root_readme.exists(), "crate root must expose the single allowed README.md");

    for file in &markdown_files {
        if file == &root_readme {
            continue;
        }
        assert!(
            file.starts_with(root.join("docs")),
            "non-root markdown must live under docs/: {}",
            file.display()
        );
    }

    let mut docs: Vec<_> = markdown_files
        .iter()
        .filter(|file| file.starts_with(root.join("docs")))
        .filter_map(|file| file.file_name().and_then(|name| name.to_str()).map(str::to_string))
        .collect();
    docs.sort();

    assert_eq!(
        docs,
        [
            "ARCHITECTURE.md",
            "BOUNDARY.md",
            "CHANGE_RULES.md",
            "COMMANDS.md",
            "DECISIONS.md",
            "DETERMINISM.md",
            "FAILURE_HANDLING.md",
            "PUBLIC_API.md",
            "REPORT_CONTRACT.md",
            "TESTS.md",
        ],
        "docs/ must stay at the 10-file allowance"
    );
}

fn collect_markdown(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_markdown_inner(root, &mut files);
    files
}

fn collect_markdown_inner(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("fixtures" | "snapshots")
            ) {
                continue;
            }
            collect_markdown_inner(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.push(path);
        }
    }
}
