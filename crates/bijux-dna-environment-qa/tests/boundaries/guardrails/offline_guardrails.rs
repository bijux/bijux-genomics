use std::fs;
use std::path::PathBuf;

#[test]
fn offline_policy_is_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("EFFECTS.md");
    let content = fs::read_to_string(&doc)
        .unwrap_or_else(|err| panic!("read EFFECTS.md at {}: {err}", doc.display()));
    assert!(
        content.to_lowercase().contains("offline"),
        "EFFECTS.md must state offline-by-default behavior"
    );
}
