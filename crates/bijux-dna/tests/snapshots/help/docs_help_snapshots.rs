use std::fs;

#[path = "../../support.rs"]
mod support;

#[test]
fn cli_help_texts_are_documented() {
    let doc = support::crate_root("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("docs")
        .join("COMMANDS.md");
    let content = fs::read_to_string(&doc).expect("read COMMANDS.md");
    for cmd in ["bijux-dna plan", "bijux-dna execute", "bijux-dna dry-run"] {
        assert!(content.contains(cmd), "COMMANDS.md must include {cmd}");
    }
}
