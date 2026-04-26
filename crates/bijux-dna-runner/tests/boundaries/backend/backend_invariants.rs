use bijux_dna_core::prelude::CommandSpecV1;

#[test]
fn backend_invariants_are_documented() {
    let doc = std::fs::read_to_string(
        crate::support::crate_root("bijux-dna-runner")
            .unwrap_or_else(|err| panic!("resolve runner root: {err}"))
            .join("docs")
            .join("EXECUTION_SPEC.md"),
    )
    .unwrap_or_else(|err| panic!("docs/EXECUTION_SPEC.md missing: {err}"));
    for phrase in ["cwd", "mount", "env", "stdout", "stderr", "exit"] {
        assert!(doc.contains(phrase), "EXECUTION_SPEC.md missing {phrase}");
    }
}

#[test]
fn command_spec_is_stable() {
    let spec = CommandSpecV1 { template: vec!["fastp".to_string(), "-h".to_string()] };
    let json1 = serde_json::to_string(&spec).unwrap_or_else(|err| panic!("serialize: {err}"));
    let json2 = serde_json::to_string(&spec).unwrap_or_else(|err| panic!("serialize: {err}"));
    assert_eq!(json1, json2);
}
