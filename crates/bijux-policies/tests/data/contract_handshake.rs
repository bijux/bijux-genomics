#[path = "../support/fs.rs"]
mod support;

use serde_json::Value;

#[test]
fn contract_handshake_fixture_shapes() {
    let root = support::workspace_root();
    let fixtures = [
        root.join("crates/bijux-policies/tests/fixtures/handshake/plan.json"),
        root.join("crates/bijux-policies/tests/fixtures/handshake/manifest.json"),
        root.join("crates/bijux-policies/tests/fixtures/handshake/report.json"),
    ];
    let mut missing = Vec::new();
    for fixture in fixtures {
        if !fixture.exists() {
            missing.push(fixture.display().to_string());
            continue;
        }
        let raw = support::read_to_string(&fixture);
        let _: Value = serde_json::from_str(&raw).expect("fixture JSON parse");
    }

    assert!(
        missing.is_empty(),
        "Contract handshake fixtures are missing.\n\
Add fixtures under crates/bijux-policies/tests/fixtures/handshake.\n\
See docs/STYLE.md for fixture guidance.\n\
Missing:\n{}",
        missing.join("\n")
    );
}
