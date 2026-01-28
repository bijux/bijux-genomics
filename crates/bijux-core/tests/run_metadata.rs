use bijux_core::{RunMetadataV1, ToolInvocationV1};

#[test]
fn run_metadata_v1_requires_fields() {
    let json = "{}";
    assert!(serde_json::from_str::<RunMetadataV1>(json).is_err());
}

#[test]
fn tool_invocation_v1_requires_fields() {
    let json = "{}";
    assert!(serde_json::from_str::<ToolInvocationV1>(json).is_err());
}
