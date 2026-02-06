use anyhow::Result;
use bijux_core::metrics::ToolInvocationV1;
use bijux_core::parameters_json_canonicalization;
use bijux_core::params_hash;
use std::collections::BTreeMap;

#[test]
fn tool_invocation_roundtrip_and_hash_stability() -> Result<()> {
    let params_a = serde_json::json!({
        "beta": 1.0,
        "alpha": {
            "x": 2,
            "y": 3.5
        }
    });
    let params_b = serde_json::json!({
        "alpha": {
            "y": 3.5,
            "x": 2
        },
        "beta": 1.0
    });

    let canonical_a = parameters_json_canonicalization(&params_a);
    let canonical_b = parameters_json_canonicalization(&params_b);
    let hash_a = params_hash(&canonical_a)?;
    let hash_b = params_hash(&canonical_b)?;
    assert_eq!(hash_a, hash_b);

    let invocation = ToolInvocationV1 {
        schema_version: "bijux.tool_invocation.v1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        resolved_tool_version: Some("0.23.4".to_string()),
        image_digest: "sha256:abc".to_string(),
        runner_kind: "docker".to_string(),
        platform: "local".to_string(),
        parameters_json: canonical_a.clone(),
        parameters_json_normalized: canonical_a.clone(),
        effective_params_json: serde_json::json!({}),
        effective_params_json_normalized: serde_json::json!({}),
        params_provenance: serde_json::json!({
            "tool_params": canonical_a.clone(),
            "defaults": serde_json::json!({}),
            "overrides": serde_json::json!({}),
            "effective_params": serde_json::json!({}),
        }),
        params_provenance_normalized: serde_json::json!({}),
        adapter_bank: None,
        banks: None,
        bank_assets: None,
        resources: bijux_core::contract::ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        environment: BTreeMap::new(),
        input_hashes: vec!["ih".to_string()],
        output_hashes: vec!["oh".to_string()],
        executed_command: None,
    };
    let encoded = serde_json::to_string(&invocation)?;
    let decoded: ToolInvocationV1 = serde_json::from_str(&encoded)?;
    let hash_roundtrip = params_hash(&decoded.parameters_json)?;
    assert_eq!(hash_a, hash_roundtrip);
    Ok(())
}
