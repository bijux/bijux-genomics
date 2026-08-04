use anyhow::{Context, Result};

fn repository_root() -> Result<std::path::PathBuf> {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(std::path::Path::to_path_buf)
        .context("environment crate is outside the repository workspace")
}

#[test]
fn checked_in_container_publications_are_admitted_by_environment_types() -> Result<()> {
    let root = repository_root()?;
    let inventory = bijux_dna_environment::evidence::load_image_inventory_publication(
        &root.join("containers/reports/image-inventory.json"),
    )?;
    let runtime = bijux_dna_environment::evidence::load_runtime_verification_publication(
        &root.join("containers/reports/runtime-verification.json"),
    )?;

    assert_eq!(inventory.tools.len(), 89);
    assert_eq!(
        inventory
            .tools
            .iter()
            .filter(|tool| matches!(
                tool.support_disposition,
                bijux_dna_environment::evidence::ContainerSupportDisposition::Retained
            ))
            .count(),
        85
    );
    assert_eq!(runtime.tools.len(), 85);
    assert_eq!(runtime.catalog.sha256, inventory.catalog_sha256);
    assert_eq!(runtime.inventory.sha256, inventory.publication.raw_sha256);
    assert_eq!(runtime.repository_commit, inventory.repository_commit);
    Ok(())
}

#[test]
fn checked_in_publication_schemas_are_owned_by_environment_types() {
    let inventory = bijux_dna_environment::evidence::image_inventory_publication_schema();
    let runtime = bijux_dna_environment::evidence::runtime_verification_publication_schema();
    let tool_contract = bijux_dna_environment::evidence::tool_contract::tool_contract_schema();
    assert_eq!(inventory["title"], "ContainerImageInventoryPublication");
    assert_eq!(runtime["title"], "RuntimeVerificationPublication");
    assert_eq!(tool_contract["title"], "ContainerToolContract");
}

#[test]
fn publication_admission_rejects_identity_drift() -> Result<()> {
    let root = repository_root()?;
    let mut inventory = bijux_dna_environment::evidence::load_image_inventory_publication(
        &root.join("containers/reports/image-inventory.json"),
    )?;
    inventory.tools[0].selected_image_sha256 = Some("0".repeat(64));
    let inventory_error =
        bijux_dna_environment::evidence::validate_image_inventory_publication(&inventory)
            .err()
            .map(|error| error.to_string());
    assert!(inventory_error
        .as_deref()
        .is_some_and(|message| message.contains("not a SHA-256 digest")));

    let mut runtime = bijux_dna_environment::evidence::load_runtime_verification_publication(
        &root.join("containers/reports/runtime-verification.json"),
    )?;
    runtime.catalog.sha256 = "not-a-digest".to_string();
    let runtime_error =
        bijux_dna_environment::evidence::validate_runtime_verification_publication(&runtime)
            .err()
            .map(|error| error.to_string());
    assert!(runtime_error
        .as_deref()
        .is_some_and(|message| message.contains("not a SHA-256 digest")));
    Ok(())
}

#[test]
fn checked_in_publication_schemas_match_rust_types() -> Result<()> {
    let root = repository_root()?;
    let inventory_path =
        root.join("containers/contracts/schemas/image-inventory-publication.v1.json");
    let runtime_path =
        root.join("containers/contracts/schemas/runtime-verification-publication.v1.json");
    let tool_contract_path = root.join("containers/contracts/schemas/tool-contract.v1.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        bijux_dna_environment::evidence::write_image_inventory_publication_schema(&inventory_path)?;
        bijux_dna_environment::evidence::write_runtime_verification_publication_schema(
            &runtime_path,
        )?;
        bijux_dna_environment::evidence::tool_contract::write_tool_contract_schema(
            &tool_contract_path,
        )?;
    }
    let inventory: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(inventory_path)?)?;
    let runtime: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(runtime_path)?)?;
    let tool_contract: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tool_contract_path)?)?;
    assert_eq!(inventory, bijux_dna_environment::evidence::image_inventory_publication_schema());
    assert_eq!(runtime, bijux_dna_environment::evidence::runtime_verification_publication_schema());
    assert_eq!(
        tool_contract,
        bijux_dna_environment::evidence::tool_contract::tool_contract_schema()
    );
    Ok(())
}
