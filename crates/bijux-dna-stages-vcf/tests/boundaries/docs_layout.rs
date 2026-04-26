use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const ALLOWED_DOCS: &[&str] = &[
    "ARCHITECTURE.md",
    "BOUNDARY.md",
    "CHANGE_RULES.md",
    "COMMANDS.md",
    "DEPENDENCIES.md",
    "EFFECTS.md",
    "INDEX.md",
    "PUBLIC_API.md",
    "STAGE_CONTRACTS.md",
    "TESTS.md",
];

#[test]
fn markdown_docs_stay_in_root_readme_or_docs_dir() {
    let root = crate_root();
    let markdown_files = markdown_files(&root);
    let expected = expected_markdown_files(&root);

    assert_eq!(
        markdown_files, expected,
        "crate docs must be exactly root README.md plus the allowed docs directory files"
    );
}

#[test]
fn docs_index_references_every_allowed_doc() {
    let root = crate_root();
    let index = std::fs::read_to_string(root.join("docs/INDEX.md"))
        .unwrap_or_else(|err| panic!("read docs/INDEX.md: {err}"));

    for doc in ALLOWED_DOCS {
        if *doc == "INDEX.md" {
            continue;
        }

        assert!(index.contains(doc), "docs/INDEX.md must reference docs/{doc}");
    }
}

fn crate_root() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let actual = root.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    assert_eq!(actual, "bijux-dna-stages-vcf", "unexpected crate root");
    root
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

    for doc in &expected {
        assert!(
            root.join(doc).is_file(),
            "expected documentation file must exist: {}",
            doc.display()
        );
    }

    expected
}
