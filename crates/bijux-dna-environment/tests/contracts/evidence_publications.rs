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
fn evidence_chain_rejects_cross_publication_identity_and_roster_drift() -> Result<()> {
    let root = repository_root()?;
    let catalog = bijux_dna_environment::evidence::load_container_catalog(&root)?;
    let inventory = bijux_dna_environment::evidence::load_image_inventory_publication(
        &root.join("containers/reports/image-inventory.json"),
    )?;
    let runtime = bijux_dna_environment::evidence::load_runtime_verification_publication(
        &root.join("containers/reports/runtime-verification.json"),
    )?;
    bijux_dna_environment::evidence::validate_container_evidence_chain(
        &inventory.catalog_sha256,
        &catalog,
        &inventory,
        &runtime,
    )?;

    let mut stale_runtime = runtime.clone();
    stale_runtime.inventory.sha256 = "d".repeat(64);
    assert!(bijux_dna_environment::evidence::validate_container_evidence_chain(
        &inventory.catalog_sha256,
        &catalog,
        &inventory,
        &stale_runtime,
    )
    .is_err_and(|error| error.to_string().contains("admitted raw inventory")));

    let mut incomplete_inventory = inventory.clone();
    incomplete_inventory.tools.pop();
    refresh_inventory_summary(&mut incomplete_inventory);
    assert!(bijux_dna_environment::evidence::validate_container_evidence_chain(
        &inventory.catalog_sha256,
        &catalog,
        &incomplete_inventory,
        &runtime,
    )
    .is_err_and(|error| error.to_string().contains("tool set differs")));

    let mut capability_drift = inventory.clone();
    capability_drift.tools[0].apptainer_supported = !capability_drift.tools[0].apptainer_supported;
    assert!(bijux_dna_environment::evidence::validate_container_inventory_catalog(
        &inventory.catalog_sha256,
        &catalog,
        &capability_drift,
    )
    .is_err_and(|error| error.to_string().contains("identity differs")));
    Ok(())
}

#[test]
fn recipe_evidence_chain_recomputes_selected_image_freshness() -> Result<()> {
    let root = repository_root()?;
    let commit = bijux_dna_environment::evidence::repository_head_commit(&root)?;
    let mut catalog = bijux_dna_environment::evidence::load_container_catalog(&root)?;
    catalog.tools.retain(|tool| tool.id == "ibdhap");
    let tool = catalog.tools.first().context("catalog omits ibdhap")?;
    let recipe = bijux_dna_environment::evidence::apptainer_recipe_identity_at_commit(
        &root,
        &commit,
        &tool.apptainer_definition,
    )?;

    let mut inventory = bijux_dna_environment::evidence::load_image_inventory_publication(
        &root.join("containers/reports/image-inventory.json"),
    )?;
    inventory.repository_commit.clone_from(&commit);
    inventory.tools.retain(|inventory_tool| inventory_tool.tool == "ibdhap");
    refresh_inventory_summary(&mut inventory);
    let inventory_tool = inventory.tools.first_mut().context("inventory omits ibdhap")?;
    let selected_sha256 = inventory_tool
        .selected_image_sha256
        .clone()
        .context("ibdhap inventory has no selected image")?;
    let selected = inventory_tool
        .candidates
        .iter_mut()
        .find(|candidate| candidate.sha256 == selected_sha256)
        .context("ibdhap inventory omits selected candidate")?;
    selected.admitted = true;
    selected.current_recipe = true;
    selected.build_evidence = vec![bijux_dna_environment::evidence::ContainerBuildEvidence {
        status: bijux_dna_environment::evidence::BuildEvidenceStatus::Passed,
        definition_sha256: recipe.definition_sha256,
        repository_commit: commit.clone(),
        build_context: Some(recipe.build_context),
    }];
    bijux_dna_environment::evidence::validate_container_recipe_evidence_chain(
        &root, &commit, &catalog, &inventory,
    )?;

    inventory.tools[0]
        .candidates
        .iter_mut()
        .find(|candidate| candidate.sha256 == selected_sha256)
        .context("ibdhap inventory omits selected candidate")?
        .build_evidence[0]
        .definition_sha256 = "e".repeat(64);
    assert!(bijux_dna_environment::evidence::validate_container_recipe_evidence_chain(
        &root, &commit, &catalog, &inventory,
    )
    .is_err_and(|error| error.to_string().contains("lacks exact current recipe")));
    Ok(())
}

#[test]
fn checked_in_publication_schemas_are_owned_by_environment_types() {
    let inventory = bijux_dna_environment::evidence::image_inventory_publication_schema();
    let scan = bijux_dna_environment::evidence::image_inventory_scan_schema();
    let runtime = bijux_dna_environment::evidence::runtime_verification_publication_schema();
    let storage_plan = bijux_dna_environment::evidence::image_storage_plan_schema();
    let storage_report = bijux_dna_environment::evidence::image_storage_report_schema();
    let tool_contract = bijux_dna_environment::evidence::tool_contract::tool_contract_schema();
    assert_eq!(inventory["title"], "ContainerImageInventoryPublication");
    assert_eq!(scan["title"], "ImageInventoryScan");
    assert_eq!(runtime["title"], "RuntimeVerificationPublication");
    assert_eq!(storage_plan["title"], "ImageStoragePlan");
    assert_eq!(storage_report["title"], "ImageStorageReport");
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

    let mut incoherent = bijux_dna_environment::evidence::load_image_inventory_publication(
        &root.join("containers/reports/image-inventory.json"),
    )?;
    let selected = incoherent.tools[0]
        .selected_image_sha256
        .clone()
        .context("fixture has no selected image")?;
    let selected_candidate = incoherent.tools[0]
        .candidates
        .iter_mut()
        .find(|candidate| candidate.sha256 == selected)
        .context("fixture omits selected candidate")?;
    selected_candidate.disposition =
        bijux_dna_environment::evidence::ImageCandidateDisposition::HistoricalRunDependency;
    selected_candidate.real_run_references = selected_candidate.real_run_references.max(1);
    assert!(bijux_dna_environment::evidence::validate_image_inventory_publication(&incoherent)
        .is_err_and(|error| error.to_string().contains("unique canonical candidate")));

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

fn refresh_inventory_summary(
    inventory: &mut bijux_dna_environment::evidence::ContainerImageInventoryPublication,
) {
    let mut states = std::collections::BTreeMap::new();
    for tool in &inventory.tools {
        let state = match tool.state {
            bijux_dna_environment::evidence::ContainerImageState::Current => "current",
            bijux_dna_environment::evidence::ContainerImageState::Historical => "historical",
            bijux_dna_environment::evidence::ContainerImageState::Ambiguous => "ambiguous",
            bijux_dna_environment::evidence::ContainerImageState::Missing => "missing",
            bijux_dna_environment::evidence::ContainerImageState::Unsupported => "unsupported",
        };
        *states.entry(state.to_string()).or_insert(0) += 1;
    }
    let candidate_historical = inventory
        .tools
        .iter()
        .flat_map(|tool| &tool.candidates)
        .filter(|candidate| {
            candidate.disposition
                == bijux_dna_environment::evidence::ImageCandidateDisposition::HistoricalRunDependency
        })
        .count() as u64;
    let candidate_obsolete = inventory
        .tools
        .iter()
        .flat_map(|tool| &tool.candidates)
        .filter(|candidate| {
            candidate.disposition
                == bijux_dna_environment::evidence::ImageCandidateDisposition::UnreferencedObsolete
        })
        .count() as u64;
    let detached_obsolete = inventory
        .unknown_images
        .iter()
        .filter(|image| {
            image.disposition
                == bijux_dna_environment::evidence::DetachedImageDisposition::UnreferencedObsolete
        })
        .count() as u64;
    inventory.summary.tool_count = inventory.tools.len() as u64;
    inventory.summary.states = states;
    inventory.summary.historical_dependency_count =
        if inventory.publication.raw_schema_version == "bijux.container.image_inventory_scan.v1" {
            candidate_historical + inventory.historical_images.len() as u64
        } else {
            inventory.historical_images.len() as u64
        };
    inventory.summary.unknown_image_count = inventory.unknown_images.len() as u64;
    inventory.summary.staged_image_count = inventory.staged_images.len() as u64;
    inventory.summary.unreferenced_obsolete_count = candidate_obsolete + detached_obsolete;
}

#[test]
fn checked_in_publication_schemas_match_rust_types() -> Result<()> {
    let root = repository_root()?;
    let inventory_path =
        root.join("containers/contracts/schemas/image-inventory-publication.v2.json");
    let scan_path = root.join("containers/contracts/schemas/image-inventory-scan.v1.json");
    let runtime_path =
        root.join("containers/contracts/schemas/runtime-verification-publication.v2.json");
    let tool_identity_path =
        root.join("containers/contracts/schemas/tool-image-identity-evidence.v1.json");
    let storage_plan_path = root.join("containers/contracts/schemas/image-storage-plan.v1.json");
    let storage_report_path =
        root.join("containers/contracts/schemas/image-storage-report.v1.json");
    let qualification_registry_path =
        root.join("containers/contracts/schemas/qualification-run-registry.v1.json");
    let qualification_audit_path =
        root.join("containers/contracts/schemas/qualification-run-id-audit.v1.json");
    let tool_contract_path = root.join("containers/contracts/schemas/tool-contract.v1.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        bijux_dna_environment::evidence::write_image_inventory_publication_schema(&inventory_path)?;
        bijux_dna_environment::evidence::write_image_inventory_scan_schema(&scan_path)?;
        bijux_dna_environment::evidence::write_runtime_verification_publication_schema(
            &runtime_path,
        )?;
        bijux_dna_environment::evidence::tool_contract::write_tool_image_identity_evidence_schema(
            &tool_identity_path,
        )?;
        bijux_dna_environment::evidence::write_image_storage_plan_schema(&storage_plan_path)?;
        bijux_dna_environment::evidence::write_image_storage_report_schema(&storage_report_path)?;
        bijux_dna_environment::evidence::write_qualification_run_registry_schema(
            &qualification_registry_path,
        )?;
        bijux_dna_environment::evidence::write_qualification_run_id_audit_schema(
            &qualification_audit_path,
        )?;
        bijux_dna_environment::evidence::tool_contract::write_tool_contract_schema(
            &tool_contract_path,
        )?;
    }
    let inventory: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(inventory_path)?)?;
    let scan: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(scan_path)?)?;
    let runtime: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(runtime_path)?)?;
    let tool_identity: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tool_identity_path)?)?;
    let storage_plan: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(storage_plan_path)?)?;
    let storage_report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(storage_report_path)?)?;
    let qualification_registry: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(qualification_registry_path)?)?;
    let qualification_audit: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(qualification_audit_path)?)?;
    let tool_contract: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tool_contract_path)?)?;
    assert_eq!(inventory, bijux_dna_environment::evidence::image_inventory_publication_schema());
    assert_eq!(scan, bijux_dna_environment::evidence::image_inventory_scan_schema());
    assert_eq!(runtime, bijux_dna_environment::evidence::runtime_verification_publication_schema());
    assert_eq!(
        tool_identity,
        bijux_dna_environment::evidence::tool_contract::tool_image_identity_evidence_schema()
    );
    assert_eq!(storage_plan, bijux_dna_environment::evidence::image_storage_plan_schema());
    assert_eq!(storage_report, bijux_dna_environment::evidence::image_storage_report_schema());
    assert_eq!(
        qualification_registry,
        bijux_dna_environment::evidence::qualification_run_registry_schema()
    );
    assert_eq!(
        qualification_audit,
        bijux_dna_environment::evidence::qualification_run_id_audit_schema()
    );
    assert_eq!(
        tool_contract,
        bijux_dna_environment::evidence::tool_contract::tool_contract_schema()
    );
    Ok(())
}
