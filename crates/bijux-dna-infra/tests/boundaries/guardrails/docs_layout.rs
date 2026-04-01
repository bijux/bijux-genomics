use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn public_api_lists_the_curated_root_surface() {
    let content = fs::read_to_string(crate_root().join("PUBLIC_API.md"))
        .unwrap_or_else(|err| panic!("read PUBLIC_API.md: {err}"));
    for expected in [
        "hash_file_sha256",
        "IoError",
        "RetryPolicy",
        "RunLayoutContract",
        "temp_dir",
    ] {
        assert!(
            content.contains(expected),
            "PUBLIC_API.md must mention {expected}"
        );
    }
}

#[test]
fn architecture_doc_matches_the_current_module_tree() {
    let content = fs::read_to_string(crate_root().join("docs/ARCHITECTURE.md"))
        .unwrap_or_else(|err| panic!("read docs/ARCHITECTURE.md: {err}"));
    for expected in ["io/", "logging/", "paths/", "retry/", "run_directories/"] {
        assert!(
            content.contains(expected),
            "docs/ARCHITECTURE.md must mention {expected}"
        );
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
        assert!(
            content.contains(expected),
            "docs/TESTS.md must reference {expected}"
        );
    }
}
