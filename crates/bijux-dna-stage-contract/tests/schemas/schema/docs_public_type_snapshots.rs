use std::fs;

#[test]
fn public_type_snapshots_have_doc_anchors() {
    let doc = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
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
