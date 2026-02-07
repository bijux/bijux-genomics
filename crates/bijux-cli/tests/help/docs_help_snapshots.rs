use std::fs;
use std::path::PathBuf;

#[test]
fn help_texts_are_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("COMMANDS.md");
    let content = fs::read_to_string(&doc).expect("read COMMANDS.md");
    for cmd in ["bijux plan", "bijux execute", "bijux dry-run"] {
        assert!(content.contains(cmd), "COMMANDS.md must include {}", cmd);
    }
}
