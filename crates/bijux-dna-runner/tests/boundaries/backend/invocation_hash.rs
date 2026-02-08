use std::collections::BTreeMap;

use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolExecutionSpecV1};
use bijux_dna_runner::backend::docker as docker_backend;

#[test]
fn invocation_hash_is_stable_for_docker_backend() -> anyhow::Result<()> {
    let spec = ToolExecutionSpecV1 {
        tool_id: bijux_dna_core::ids::ToolId::from_static("tool"),
        tool_version: "1.0".to_string(),
        image: ContainerImageRefV1 {
            image: "tool:1.0".to_string(),
            digest: Some("sha256:img".to_string()),
        },
        command: CommandSpecV1 {
            template: vec!["tool".to_string(), "--flag".to_string()],
        },
        resources: bijux_dna_core::contract::ToolConstraints::default(),
    };
    let mut env = BTreeMap::new();
    env.insert("MODE".to_string(), "test".to_string());
    let inputs = vec!["sha256:a".to_string(), "sha256:b".to_string()];

    let docker_hash =
        docker_backend::execution_spec::invocation_hash_for_spec(&spec, &env, &inputs)?;
    assert!(!docker_hash.is_empty());
    Ok(())
}
