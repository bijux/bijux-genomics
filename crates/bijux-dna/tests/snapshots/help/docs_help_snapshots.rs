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
fn command_inventory_documents_canonical_commands() {
    let doc = super::support::crate_root("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("docs")
        .join("COMMANDS.md");
    let content = fs::read_to_string(&doc).expect("read COMMANDS.md");
    for cmd in [
        "bijux-dna run list-stages",
        "bijux-dna run validate-pre",
        "bijux-dna plan validate-profile",
        "bijux-dna plan profile-diff",
        "bijux-dna bench schema",
        "bijux-dna bench fastq trim-reads",
        "bijux-dna bench fastq validate-reads",
        "bijux-dna bench fastq screen-taxonomy",
        "bijux-dna bench fastq profile-reads",
        "bijux-dna vcf run",
    ] {
        assert!(content.contains(cmd), "COMMANDS.md must include {cmd}");
    }
    for stale in [
        "bijux-dna plan plan",
        "bijux-dna fastq",
        "`bijux-dna bench fastq trim`",
        "`bijux-dna bench fastq validate`",
        "`bijux-dna bench fastq screen`",
        "`bijux-dna bench fastq stats`",
    ] {
        assert!(!content.contains(stale), "COMMANDS.md must not document stale command {stale}");
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
