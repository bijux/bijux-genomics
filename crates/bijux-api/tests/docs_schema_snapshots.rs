use std::fs;
use std::path::PathBuf;

#[test]
fn schema_snapshots_are_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("API.md");
    let content = fs::read_to_string(&doc).expect("read API.md");
    assert!(
        content.contains("PlanResponse"),
        "API.md must reference PlanResponse"
    );
    assert!(
        content.contains("ExecuteResponse"),
        "API.md must reference ExecuteResponse"
    );
    assert!(
        content.contains("ExplainResponse"),
        "API.md must reference ExplainResponse"
    );
}
