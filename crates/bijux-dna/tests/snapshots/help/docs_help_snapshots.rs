#![allow(clippy::expect_used)]

use std::fs;

#[test]
fn cli_help_texts_are_documented() {
    let doc = super::support::crate_root("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("docs")
        .join("COMMANDS.md");
    let content = fs::read_to_string(&doc).expect("read COMMANDS.md");
    for cmd in [
        "bijux-dna env",
        "bijux-dna registry",
        "bijux-dna run",
        "bijux-dna plan",
        "bijux-dna bench",
        "bijux-dna status",
    ] {
        assert!(content.contains(cmd), "COMMANDS.md must include {cmd}");
    }
}

#[test]
fn dry_run_doc_uses_run_flag_surface() {
    let doc = super::support::crate_root("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("docs")
        .join("DRY_RUN.md");
    let content = fs::read_to_string(&doc).expect("read DRY_RUN.md");

    assert!(content.contains("bijux-dna run preprocess --dry-run"));
    assert!(!content.contains("bijux-dna dry-run"));
}
