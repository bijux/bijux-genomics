use anyhow::{Context, Result};
use bijux_dna_containers::evidence::tool_contract::{
    delivery::{load_benchmark_integrated_container_tool_ids, ContainerDeliveryCatalog},
    load_container_catalog, ContainerArtifactDelivery, ContainerProductSupport,
};
use std::path::PathBuf;
fn repo_root() -> PathBuf {
    std::env::var_os("BIJUX_REPO_ROOT").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(std::path::Path::parent)
                .unwrap_or_else(|| panic!("container crate below root"))
                .to_path_buf()
        },
        PathBuf::from,
    )
}
#[test]
fn benchmark_integration_keeps_external_artifacts_in_scope() -> Result<()> {
    let tool_ids = load_benchmark_integrated_container_tool_ids(&repo_root())?;

    for tool_id in ["angsd", "fastuniq", "impute5", "ngsbriggs"] {
        assert!(
            tool_ids.contains(tool_id),
            "external artifact `{tool_id}` must not be excluded by redistribution policy"
        );
    }
    Ok(())
}
#[test]
fn delivery_catalog_rejects_duplicate_blank_and_padded_tool_identities() -> Result<()> {
    let catalog = load_container_catalog(&repo_root())?;
    let mut wrong_schema = catalog.clone();
    wrong_schema.schema_version = "unsupported".to_owned();
    assert!(ContainerDeliveryCatalog::from_catalog(wrong_schema).is_err());
    let mut duplicate = catalog.clone();
    duplicate.tools.push(catalog.tools[0].clone());
    assert!(ContainerDeliveryCatalog::from_catalog(duplicate).is_err());
    for id in ["", " padded "] {
        let mut invalid = catalog.clone();
        invalid.tools[0].id = id.to_owned();
        assert!(ContainerDeliveryCatalog::from_catalog(invalid).is_err());
    }
    Ok(())
}
#[test]
fn external_contracts_fail_closed_on_policy_and_runtime_metadata_drift() -> Result<()> {
    let catalog = load_container_catalog(&repo_root())?;
    let index = catalog
        .tools
        .iter()
        .position(|tool| tool.external_artifact_supported)
        .context("governed external tool")?;
    for mutation in 0..7 {
        let mut candidate = catalog.clone();
        let tool = &mut candidate.tools[index];
        match mutation {
            0 => tool.product_support = ContainerProductSupport::Retired,
            1 => tool.redistribution_permitted = true,
            2 => tool.artifact_delivery = ContainerArtifactDelivery::RedistributableImage,
            3 => tool.domains.clear(),
            4 => tool.stage_ids.clear(),
            5 => tool.stage_ids = vec!["foreign.stage".to_owned()],
            _ => tool.version_command.clear(),
        }
        assert!(
            ContainerDeliveryCatalog::from_catalog(candidate)?
                .external_artifact_contracts()
                .is_err(),
            "mutation {mutation}"
        );
    }
    Ok(())
}
#[test]
fn delivery_bindings_use_sorted_deduplicated_runtime_contracts() -> Result<()> {
    let mut catalog = load_container_catalog(&repo_root())?;
    let tool = catalog
        .tools
        .iter_mut()
        .find(|tool| tool.external_artifact_supported)
        .context("external tool")?;
    tool.domains.push(tool.domains[0].clone());
    tool.stage_ids.push(tool.stage_ids[0].clone());
    let admitted = ContainerDeliveryCatalog::from_catalog(catalog)?;
    let contracts = admitted.external_artifact_contracts()?;
    let keys = admitted.external_artifact_binding_keys()?;
    assert!(contracts.windows(2).all(|pair| pair[0].tool_id < pair[1].tool_id));
    for contract in &contracts {
        assert!(contract.domains.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(contract.stage_ids.windows(2).all(|pair| pair[0] < pair[1]));
        for stage in &contract.stage_ids {
            assert!(keys.contains(&(stage.clone(), contract.tool_id.clone())));
        }
    }
    assert_eq!(
        keys.len(),
        contracts.iter().map(|contract| contract.stage_ids.len()).sum::<usize>()
    );
    Ok(())
}
