use anyhow::{Context, Result};
use bijux_dna_stage_contract::{
    canonical_external_artifact_tool_bindings, canonical_retained_tool_bindings,
    validate_tool_binding, BindingInputSourceV1, BindingInputV1, BindingOutputV1,
    BindingParameterV1, OutputDiscoveryV1, OutputSidecarDiscoveryV1, ToolArtifactSourceV1,
    ToolBindingV1,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

fn rejects_unknown<T: DeserializeOwned>(mut input: Value) -> Result<()> {
    serde_json::from_value::<T>(input.clone())?;
    input.as_object_mut().context("binding object")?.insert("undeclared".to_string(), json!(true));
    let Err(error) = serde_json::from_value::<T>(input) else {
        anyhow::bail!("{} silently discarded an unknown field", std::any::type_name::<T>());
    };
    assert!(error.to_string().contains("unknown field"), "{error}");
    Ok(())
}

#[test]
fn binding_transport_rejects_unknown_fields_at_every_owned_object_boundary() -> Result<()> {
    let registry = canonical_retained_tool_bindings()?;
    let binding = registry.bindings().next().context("retained binding")?;
    rejects_unknown::<ToolBindingV1>(serde_json::to_value(binding)?)?;
    rejects_unknown::<BindingInputV1>(
        json!({"name":"reads","artifact_contract_id":"reads.v1","source":"corpus","required":true}),
    )?;
    rejects_unknown::<BindingParameterV1>(json!({"name":"threads","required":true}))?;
    rejects_unknown::<BindingOutputV1>(
        json!({"name":"result","artifact_contract_id":"result.v1","discovery":{"kind":"exact_path"}}),
    )?;
    rejects_unknown::<ToolArtifactSourceV1>(
        json!({"kind":"catalog_image","catalog_tool_id":"samtools"}),
    )?;
    rejects_unknown::<ToolArtifactSourceV1>(
        json!({"kind":"external_artifact","artifact_kind":"sif","version_probe":["--version"],"checksum_required":true}),
    )?;
    rejects_unknown::<OutputDiscoveryV1>(json!({"kind":"exact_path"}))?;
    rejects_unknown::<OutputDiscoveryV1>(json!({"kind":"glob","pattern":"*.json"}))?;
    rejects_unknown::<OutputDiscoveryV1>(
        json!({"kind":"exact_path_with_sidecars","sidecars":[{"kind":"append_suffix","suffix":".tbi"}]}),
    )?;
    rejects_unknown::<OutputSidecarDiscoveryV1>(json!({"kind":"append_suffix","suffix":".bai"}))?;
    rejects_unknown::<OutputSidecarDiscoveryV1>(
        json!({"kind":"replace_extension","extension":"bai"}),
    )?;
    Ok(())
}

#[test]
fn all_canonical_binding_transports_round_trip_without_schema_or_payload_changes() -> Result<()> {
    for registry in
        [canonical_retained_tool_bindings()?, canonical_external_artifact_tool_bindings()?]
    {
        assert!(registry.bindings().len() > 0);
        for binding in registry.bindings() {
            let encoded = serde_json::to_value(binding)?;
            let decoded: ToolBindingV1 = serde_json::from_value(encoded.clone())?;
            validate_tool_binding(&decoded)?;
            assert_eq!(&decoded, binding);
            assert_eq!(serde_json::to_value(decoded)?, encoded);
        }
    }
    Ok(())
}

#[test]
fn legacy_input_source_remains_readable_for_evidence_admission() -> Result<()> {
    let input: BindingInputV1 = serde_json::from_value(
        json!({"name":"reads","artifact_contract_id":"reads.v1","required":true}),
    )?;
    assert_eq!(input.source, BindingInputSourceV1::LegacyUnspecified);
    let mut binding =
        canonical_retained_tool_bindings()?.bindings().next().context("binding")?.clone();
    binding.inputs[0].source = input.source;
    validate_tool_binding(&binding)?;
    Ok(())
}

#[test]
fn binding_object_schemas_declare_closed_field_sets() {
    for schema in [
        schemars::schema_for!(ToolBindingV1).to_value(),
        schemars::schema_for!(BindingInputV1).to_value(),
        schemars::schema_for!(BindingParameterV1).to_value(),
        schemars::schema_for!(BindingOutputV1).to_value(),
    ] {
        assert_eq!(schema.get("additionalProperties"), Some(&json!(false)));
    }
}
