use super::*;

#[test]
fn qualification_specification_schemas_preserve_governed_input_contracts() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (name, schema) in [
        ("qualification-specification-source.v1.json", qualification_specification_source_schema()),
        ("qualification-source-selection.v1.json", qualification_source_selection_schema()),
    ] {
        let bytes = std::fs::read(root.join("containers/contracts/schemas").join(name))?;
        let governed: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(governed, serde_json::to_value(schema)?, "{name}");
    }
    Ok(())
}
