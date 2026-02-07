use std::fs;
use std::path::PathBuf;

#[test]
fn public_type_snapshots_have_doc_anchors() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("SCHEMAS.md");
    let content = fs::read_to_string(&doc).expect("read SCHEMAS.md");

    for anchor in ["StagePlanV1", "ExecutionPlanV1", "StagePluginOutputV1"] {
        assert!(
            content.contains(anchor),
            "SCHEMAS.md must include example for {}",
            anchor
        );
    }
}
