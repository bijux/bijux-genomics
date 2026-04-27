use std::fs;
use std::path::Path;

#[test]
fn command_inventory_documents_no_runtime_commands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = root.join("docs").join("COMMANDS.md");
    let content = fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    assert!(
        content.contains("This crate owns no runtime CLI commands."),
        "COMMANDS.md must make the command ownership boundary explicit"
    );
    assert!(content.contains("None."), "COMMANDS.md must list the managed command set as empty");
    assert!(
        !root.join("src").join("bin").exists(),
        "bijux-dna-pipelines must remain a library crate without src/bin command entrypoints"
    );
}
