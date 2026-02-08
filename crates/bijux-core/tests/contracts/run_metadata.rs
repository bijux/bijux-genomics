use bijux_core::prelude::{
    RunMetadataV1, StageMetadataV1, ToolExecutionMetadataV1, ToolInvocationMetadataV1,
};

#[test]
fn run_metadata_v1_requires_fields() {
    let json = "{}";
    assert!(serde_json::from_str::<RunMetadataV1>(json).is_err());
}

#[test]
fn tool_invocation_v1_requires_fields() {
    let json = "{}";
    assert!(serde_json::from_str::<ToolInvocationMetadataV1>(json).is_err());
}

#[test]
fn stage_metadata_v1_requires_fields() {
    let json = "{}";
    assert!(serde_json::from_str::<StageMetadataV1>(json).is_err());
}

#[test]
fn tool_execution_metadata_v1_requires_fields() {
    let json = "{}";
    assert!(serde_json::from_str::<ToolExecutionMetadataV1>(json).is_err());
}
