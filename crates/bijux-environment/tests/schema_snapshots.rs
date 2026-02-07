#[test]
fn platform_spec_schema_snapshot() {
    let spec = bijux_environment::resolve::PlatformSpec {
        name: "docker-mac-arm64".to_string(),
        runner: bijux_environment::resolve::RunnerKind::Docker,
        container_dir: "containers/docker/arm64".into(),
        image_prefix: "bijuxdna".to_string(),
        arch: "arm64".to_string(),
    };
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&spec).expect("canonical"),
    )
    .expect("utf8");
    let expected = include_str!("fixtures/env_schema/platform_spec.json");
    assert_eq!(actual, expected);
}

#[test]
fn tool_image_spec_schema_snapshot() {
    let spec = bijux_environment::resolve::ToolImageSpec {
        tool: "fastp".to_string(),
        version: "0.23.4".to_string(),
        arch: "arm64".to_string(),
        digest: Some("sha256:img".to_string()),
    };
    let actual = String::from_utf8(
        bijux_core::contract::canonical::to_canonical_json_bytes(&spec).expect("canonical"),
    )
    .expect("utf8");
    let expected = include_str!("fixtures/env_schema/tool_image_spec.json");
    assert_eq!(actual, expected);
}
