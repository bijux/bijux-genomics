use std::fs;
use std::path::PathBuf;

#[test]
fn offline_policy_is_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("OFFLINE_POLICY.md");
    let content = fs::read_to_string(&doc)
        .unwrap_or_else(|err| panic!("read OFFLINE_POLICY.md at {}: {err}", doc.display()));
    assert!(
        content.to_lowercase().contains("offline"),
        "OFFLINE_POLICY.md must state offline-by-default behavior"
    );
}
