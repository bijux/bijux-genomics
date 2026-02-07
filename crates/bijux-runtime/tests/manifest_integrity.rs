use bijux_runtime::run_layout::RunManifest;

#[test]
fn manifest_has_required_fields() {
    let manifest = RunManifest {
        schema_version: "bijux.run_manifest.v3".to_string(),
        contract_version: bijux_core::contract::ContractVersion { major: 1, minor: 0 },
        run_id: "run-1".to_string(),
        started_at: "2024-01-01T00:00:00Z".to_string(),
        finished_at: "2024-01-01T00:00:10Z".to_string(),
        pipeline: "fastq-to-fastq__default__v1".to_string(),
        graph_hash: "sha256:graph".to_string(),
        cache_key: None,
        layout: bijux_core::foundation::input_assessment::FastqLayout::SingleEnd,
        stages: Vec::new(),
        tool_invocations: vec![bijux_core::metrics::ToolInvocationV1 {
            schema_version: "bijux.tool_invocation.v1".to_string(),
            contract_version: bijux_core::contract::ContractVersion { major: 1, minor: 0 },
            stage_id: bijux_core::ids::StageId::new("fastq.trim"),
            tool_id: bijux_core::ids::ToolId::new("fastp"),
            tool_version: "1.0".to_string(),
            resolved_tool_version: None,
            image_digest: "sha256:img".to_string(),
            runner_kind: "docker".to_string(),
            platform: "local".to_string(),
            parameters_json: serde_json::json!({}),
            parameters_json_normalized: serde_json::json!({}),
            effective_params_json: serde_json::json!({}),
            effective_params_json_normalized: serde_json::json!({}),
            params_provenance: serde_json::json!({}),
            params_provenance_normalized: serde_json::json!({}),
            adapter_bank: None,
            banks: None,
            bank_assets: None,
            resources: bijux_core::contract::ToolConstraints::default(),
            environment: Default::default(),
            input_hashes: vec!["sha256:input".to_string()],
            output_hashes: vec!["sha256:output".to_string()],
            executed_command: None,
        }],
        artifacts: Vec::new(),
    };
    let json = serde_json::to_value(&manifest).expect("serialize");
    for key in [
        "schema_version",
        "contract_version",
        "run_id",
        "started_at",
        "finished_at",
        "pipeline",
        "graph_hash",
        "layout",
        "stages",
        "tool_invocations",
        "artifacts",
    ] {
        assert!(json.get(key).is_some(), "missing {key}");
    }
    let invocations = json
        .get("tool_invocations")
        .and_then(|value| value.as_array())
        .expect("tool_invocations array");
    assert!(!invocations.is_empty(), "tool_invocations empty");
    let first = &invocations[0];
    assert!(
        first.get("input_hashes").and_then(|value| value.as_array()).is_some(),
        "tool_invocation missing input_hashes"
    );
}
