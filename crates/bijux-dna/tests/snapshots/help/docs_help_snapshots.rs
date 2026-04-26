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
