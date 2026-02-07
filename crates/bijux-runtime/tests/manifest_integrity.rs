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
        tool_invocations: Vec::new(),
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
}
