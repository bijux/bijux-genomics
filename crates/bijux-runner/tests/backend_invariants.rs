use bijux_core::foundation::CommandSpecV1;

#[test]
fn backend_invariants_are_documented() {
    let doc = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("BACKENDS.md"),
    )
    .expect("BACKENDS.md missing");
    for phrase in [
        "cwd",
        "mount",
        "env",
        "stdout",
        "stderr",
        "exit",
    ] {
        assert!(doc.contains(phrase), "BACKENDS.md missing {phrase}");
    }
}

#[test]
fn command_spec_is_stable() {
    let spec = CommandSpecV1 {
        tool_id: bijux_core::ids::ToolId::new("fastp"),
        template: "fastp".to_string(),
        args: vec!["-h".to_string()],
        working_dir: None,
    };
    let json1 = serde_json::to_string(&spec).expect("serialize");
    let json2 = serde_json::to_string(&spec).expect("serialize");
    assert_eq!(json1, json2);
}
