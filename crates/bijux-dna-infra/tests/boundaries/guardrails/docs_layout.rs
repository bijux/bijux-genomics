use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn public_api_lists_the_curated_root_surface() {
    let content = fs::read_to_string(crate_root().join("docs/PUBLIC_API.md"))
        .unwrap_or_else(|err| panic!("read docs/PUBLIC_API.md: {err}"));
    for expected in ["hash_file_sha256", "IoError", "RetryPolicy", "RunLayoutContract", "temp_dir"]
    {
        assert!(content.contains(expected), "PUBLIC_API.md must mention {expected}");
    }
}

#[test]
fn architecture_doc_matches_the_current_module_tree() {
    let content = fs::read_to_string(crate_root().join("docs/ARCHITECTURE.md"))
        .unwrap_or_else(|err| panic!("read docs/ARCHITECTURE.md: {err}"));
    for expected in ["io/", "logging/", "paths/", "retry/", "run_directories/"] {
        assert!(content.contains(expected), "docs/ARCHITECTURE.md must mention {expected}");
    }
}

#[test]
fn tests_doc_references_the_active_test_files() {
    let content = fs::read_to_string(crate_root().join("docs/TESTS.md"))
        .unwrap_or_else(|err| panic!("read docs/TESTS.md: {err}"));
    for expected in [
        "tests/contracts/io.rs",
        "tests/contracts/run_layout.rs",
        "tests/determinism/retry.rs",
        "tests/boundaries/guardrails/docs_layout.rs",
    ] {
        assert!(content.contains(expected), "docs/TESTS.md must reference {expected}");
    }
}

#[test]
fn commands_doc_declares_library_only_inventory() {
    let content = fs::read_to_string(crate_root().join("docs/COMMANDS.md"))
        .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));
    assert!(content.contains("library-only"), "COMMANDS.md must declare library-only scope");
    assert!(
        content.contains("## Managed Command Inventory") && content.contains("None."),
        "COMMANDS.md must list an empty managed command inventory"
    );
    assert!(
        !crate_root().join("src/bin").exists(),
        "infra must not add binary entrypoints without updating COMMANDS.md"
    );
}

#[test]
fn markdown_docs_stay_in_the_governed_locations() {
    let root = crate_root();
    let docs_dir = root.join("docs");
    let mut markdown_files = Vec::new();
    collect_markdown_files(&root, &mut markdown_files);

    let allowed_docs = [
        "ARCHITECTURE.md",
        "BOUNDARY.md",
        "COMMANDS.md",
        "DEPENDENCIES.md",
        "EFFECTS.md",
        "FORMATS.md",
        "INDEX.md",
        "PATHS.md",
        "PUBLIC_API.md",
        "TESTS.md",
    ];

    let mut offenders = Vec::new();
    for file in &markdown_files {
        let allowed_root_readme = file == &root.join("README.md");
        let allowed_doc = allowed_docs.iter().any(|name| file == &docs_dir.join(name));
        if !allowed_root_readme && !allowed_doc {
            offenders.push(file.strip_prefix(&root).unwrap_or(file).display().to_string());
        }
    }

    assert!(offenders.is_empty(), "unexpected markdown files:\n{}", offenders.join("\n"));

    let mut actual_docs = fs::read_dir(&docs_dir)
        .unwrap_or_else(|err| panic!("read docs dir: {err}"))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read docs entry: {err}")).file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    actual_docs.sort();
    let expected_docs = allowed_docs.iter().map(ToString::to_string).collect::<Vec<_>>();
    assert_eq!(actual_docs, expected_docs, "docs/ must stay at the 10-file allowance");
}

fn collect_markdown_files(dir: &std::path::Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|err| panic!("read {}: {err}", dir.display())) {
        let entry = entry.unwrap_or_else(|err| panic!("read entry under {}: {err}", dir.display()));
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some("target") {
            continue;
        }
        if entry
            .file_type()
            .unwrap_or_else(|err| panic!("file type {}: {err}", path.display()))
            .is_dir()
        {
            collect_markdown_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.push(path);
        }
    }
}
