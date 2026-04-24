use std::fs;

#[test]
fn schema_snapshots_are_documented() {
    let doc = crate::support::crate_root("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("docs")
        .join("API.md");
    let content = fs::read_to_string(&doc)
        .unwrap_or_else(|err| panic!("read API.md at {}: {err}", doc.display()));
    assert!(content.contains("PlanResponse"), "API.md must reference PlanResponse");
    assert!(content.contains("ExecuteResponse"), "API.md must reference ExecuteResponse");
    assert!(content.contains("ExplainResponse"), "API.md must reference ExplainResponse");
}
