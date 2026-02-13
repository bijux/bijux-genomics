#[test]
fn platform_spec_schema_snapshot() {
    let spec = bijux_dna_environment::resolve::PlatformSpec {
        name: "docker-mac-arm64".to_string(),
        runner: bijux_dna_environment::resolve::RuntimeKind::Docker,
        container_dir: "containers/docker/arm64".into(),
        image_prefix: "bijuxdna".to_string(),
        arch: "arm64".to_string(),
    };
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&spec)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    let expected = include_str!("../../fixtures/env_schema/default/platform_spec.json");
    assert_eq!(actual, expected);
}

#[test]
fn tool_image_spec_schema_snapshot() {
    let spec = bijux_dna_environment::resolve::ToolImageSpec {
        tool: "fastp".to_string(),
        version: "0.23.4".to_string(),
        digest: Some("sha256:img".to_string()),
        enabled: None,
        shipping_policy: None,
    };
    let actual = String::from_utf8(
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&spec)
            .unwrap_or_else(|err| panic!("canonical: {err}")),
    )
    .unwrap_or_else(|err| panic!("utf8: {err}"));
    let expected = include_str!("../../fixtures/env_schema/default/tool_image_spec.json");
    assert_eq!(actual, expected);
}
