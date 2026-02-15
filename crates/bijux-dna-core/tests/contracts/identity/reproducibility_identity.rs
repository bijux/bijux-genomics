use bijux_dna_core::prelude::ReproducibilityIdentityV1;

#[test]
fn reproducibility_identity_changes_when_tuple_changes() {
    let base = ReproducibilityIdentityV1 {
        image_digest: "sha256:img".to_string(),
        tool_version: "1.0.0".to_string(),
        params_hash: "sha256:params".to_string(),
        input_hash: "sha256:input".to_string(),
        bank_hashes: serde_json::json!({
            "adapter_bank_hash": "sha256:adapter",
            "reference_bank_hash": "sha256:reference",
            "taxonomy_db_hash": "sha256:taxonomy",
            "taxonomy_db_version": "2024.01",
        }),
    };
    let base_id = base.as_string();

    let mut changed = base.clone();
    changed.image_digest = "sha256:img2".to_string();
    assert_ne!(base_id, changed.as_string());

    let mut changed = base.clone();
    changed.tool_version = "1.0.1".to_string();
    assert_ne!(base_id, changed.as_string());

    let mut changed = base.clone();
    changed.params_hash = "sha256:params2".to_string();
    assert_ne!(base_id, changed.as_string());

    let mut changed = base.clone();
    changed.input_hash = "sha256:input2".to_string();
    assert_ne!(base_id, changed.as_string());

    let mut changed = base;
    changed.bank_hashes["taxonomy_db_version"] = serde_json::Value::String("2024.02".to_string());
    assert_ne!(base_id, changed.as_string());
}
