use std::fs;

use anyhow::{Context, Result};
use bijux_dna_containers::api::{
    derive_qualification_run_registry, load_container_catalog, load_qualification_run_registry,
    render_container_run_index_json, render_container_run_index_markdown,
    render_qualification_run_registry, validate_container_run_index,
    validate_qualification_run_registry, ContainerArtifactDelivery, ContainerProductSupport,
    ContainerRunIndex, ContainerRunIndexSource, QualificationRegistryDerivationRequest,
    QualificationRunState, CONTAINER_RUN_INDEX_SCHEMA_V1,
};

#[test]
fn registry_closes_all_catalog_tools_without_reusing_historical_ids() -> Result<()> {
    let root = repository_root()?;
    let request = request(&root);
    let first = derive_qualification_run_registry(&request)?;
    let second = derive_qualification_run_registry(&request)?;

    assert_eq!(first.registry, second.registry);
    assert_eq!(first.audit, second.audit);
    assert_eq!(first.registry.tools.len(), 89);
    assert_eq!(first.registry.runs.len(), 159);
    assert_eq!(first.registry.summary.run_count, 159);
    assert_eq!(first.registry.summary.campaign_count, 1);
    assert_eq!(first.registry.summary.active_count, 68);
    assert_eq!(first.registry.summary.allocated_count, 21);
    assert_eq!(first.registry.summary.historical_count, 67);
    assert_eq!(first.registry.summary.reserved_count, 2);
    assert_eq!(first.registry.runs[0].run_id, "run-1000");
    assert_eq!(first.registry.runs[0].state, QualificationRunState::Campaign);
    assert_eq!(first.registry.runs[0].tool_ids.len(), 85);
    assert_eq!(first.registry.runs[81].run_id, "run-1081");
    assert_eq!(first.registry.runs[81].state, QualificationRunState::Reserved);
    assert_eq!(first.registry.runs[85].run_id, "run-1085");
    assert_eq!(first.registry.runs[85].state, QualificationRunState::Reserved);
    assert_eq!(first.registry.runs[158].run_id, "run-1158");
    assert_eq!(first.registry.runs[158].citation.citation_key, "bijux-container-run-1158");
    assert_eq!(first.audit.supported_tool_count, 89);
    assert_eq!(first.audit.redistributable_tool_count, 85);
    assert_eq!(first.audit.external_artifact_tool_count, 4);
    assert_eq!(first.audit.allocated_mapping_count, 18);
    assert_eq!(first.audit.historical_identities.len(), 138);
    assert_eq!(first.audit.reserved_run_count, 2);
    assert_eq!(first.audit.reserved_run_ids.len(), 2);
    assert_eq!(first.audit.reserved_run_ids[0].run_id, "run-1081");
    assert_eq!(first.audit.reserved_run_ids[1].run_id, "run-1085");
    assert_eq!(first.audit.historical_max_run_number, 1140);
    assert_eq!(run_id(&first, "adapterremoval"), Some("run-1002"));
    assert_eq!(run_id(&first, "bijux_dna"), Some("run-1141"));
    assert_eq!(run_id(&first, "yleaf"), Some("run-1157"));
    assert_eq!(run_id(&first, "angsd"), Some("run-1094"));
    assert_eq!(run_id(&first, "fastuniq"), Some("run-1023"));
    assert_eq!(run_id(&first, "impute5"), Some("run-1158"));
    assert_eq!(run_id(&first, "ngsbriggs"), Some("run-1046"));
    for tool_id in ["angsd", "fastuniq", "impute5", "ngsbriggs"] {
        let entry = first
            .registry
            .tools
            .iter()
            .find(|candidate| candidate.tool_id == tool_id)
            .context("external registry entry")?;
        assert_eq!(entry.product_support, ContainerProductSupport::Supported);
        assert_eq!(entry.artifact_delivery, ContainerArtifactDelivery::ExternalArtifact);
        assert!(!entry.redistribution_permitted);
        let external_run_id = run_id(&first, tool_id).context("external run ID")?;
        let run = first
            .registry
            .runs
            .iter()
            .find(|candidate| candidate.run_id == external_run_id)
            .context("external run definition")?;
        assert_eq!(run.state, QualificationRunState::Allocated);
        assert!(run.citation.limitations.iter().any(|limitation| {
            limitation == "The protocol has no selected publication until qualification completes."
        }));
    }

    let rendered = render_qualification_run_registry(&first.registry)?;
    assert_eq!(rendered, render_qualification_run_registry(&second.registry)?);
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("qualification-runs.toml");
    fs::write(&path, rendered)?;
    assert_eq!(load_qualification_run_registry(&root, &path)?, first.registry);
    Ok(())
}

#[test]
fn registry_rejects_duplicate_ids_incomplete_external_execution_and_catalog_drift() -> Result<()> {
    let root = repository_root()?;
    let derived = derive_qualification_run_registry(&request(&root))?;
    let catalog = load_container_catalog(&root)?;

    let mut duplicate = derived.registry.clone();
    let adapterremoval = run_id(&derived, "adapterremoval").context("adapterremoval run")?;
    duplicate.tools.iter_mut().find(|row| row.tool_id == "addeam").context("addeam row")?.run_id =
        Some(adapterremoval.to_string());
    assert!(validate_qualification_run_registry(&catalog, &duplicate).is_err());

    let mut external = derived.registry.clone();
    external
        .tools
        .iter_mut()
        .find(|row| row.tool_id == "angsd")
        .context("angsd row")?
        .resource_profile = None;
    assert!(validate_qualification_run_registry(&catalog, &external).is_err());

    let mut support_drift = derived.registry.clone();
    support_drift
        .tools
        .iter_mut()
        .find(|row| row.tool_id == "angsd")
        .context("angsd row")?
        .product_support = ContainerProductSupport::Retired;
    assert!(validate_qualification_run_registry(&catalog, &support_drift).is_err());

    let mut delivery_drift = derived.registry.clone();
    delivery_drift
        .tools
        .iter_mut()
        .find(|row| row.tool_id == "angsd")
        .context("angsd row")?
        .artifact_delivery = ContainerArtifactDelivery::RedistributableImage;
    assert!(validate_qualification_run_registry(&catalog, &delivery_drift).is_err());

    let mut redistribution_drift = derived.registry.clone();
    redistribution_drift
        .tools
        .iter_mut()
        .find(|row| row.tool_id == "angsd")
        .context("angsd row")?
        .redistribution_permitted = true;
    assert!(validate_qualification_run_registry(&catalog, &redistribution_drift).is_err());

    let mut stage_drift = derived.registry;
    stage_drift
        .tools
        .iter_mut()
        .find(|row| row.tool_id == "adapterremoval")
        .context("adapterremoval row")?
        .stage_id = "fastq.trim_reads".to_string();
    assert!(validate_qualification_run_registry(&catalog, &stage_drift).is_err());

    let mut missing_campaign = derive_qualification_run_registry(&request(&root))?.registry;
    missing_campaign.runs.remove(0);
    assert!(validate_qualification_run_registry(&catalog, &missing_campaign).is_err());

    let mut reused_tombstone = derive_qualification_run_registry(&request(&root))?.registry;
    reused_tombstone.runs[81].state = QualificationRunState::Active;
    reused_tombstone.runs[81].tool_ids = vec!["adapterremoval".to_string()];
    assert!(validate_qualification_run_registry(&catalog, &reused_tombstone).is_err());
    Ok(())
}

#[test]
fn paper_facing_index_preserves_complete_namespace_and_missing_evidence() -> Result<()> {
    let root = repository_root()?;
    let derived = derive_qualification_run_registry(&request(&root))?;
    let registry_bytes = render_qualification_run_registry(&derived.registry)?.into_bytes();
    let index = ContainerRunIndex {
        schema_version: CONTAINER_RUN_INDEX_SCHEMA_V1.to_string(),
        source_registry: ContainerRunIndexSource {
            uri: "repo://containers/contracts/qualification-runs.toml".to_string(),
            sha256: bijux_dna_infra::hash_bytes_sha256(&registry_bytes),
            size_bytes: u64::try_from(registry_bytes.len())?,
        },
        catalog_sha256: derived.registry.catalog_sha256.clone(),
        historical_max_run_number: derived.registry.historical_max_run_number,
        namespace: derived.registry.namespace.clone(),
        summary: derived.registry.summary.clone(),
        runs: derived.registry.runs.clone(),
    };

    validate_container_run_index(&index)?;
    assert!(render_container_run_index_json(&index)?.contains("\"run_id\": \"run-1158\""));
    assert!(render_container_run_index_markdown(&index).contains("| `run-1158` |"));

    let mut incomplete = index.clone();
    incomplete.runs.pop();
    assert!(validate_container_run_index(&incomplete).is_err());

    let mut hidden_missing_evidence = index;
    let reserved = hidden_missing_evidence
        .runs
        .iter_mut()
        .find(|run| run.citation.evidence_uris.is_empty())
        .context("run without selected evidence")?;
    reserved.citation.limitations.clear();
    assert!(validate_container_run_index(&hidden_missing_evidence).is_err());
    Ok(())
}

#[test]
fn derivation_rejects_cross_tool_history_and_nonpassing_final_runs() -> Result<()> {
    let root = repository_root()?;
    let directory = tempfile::tempdir()?;
    let ledger_path = directory.path().join("attempt-ledger.json");
    let mut ledger: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("containers/runs/attempt-ledger.json"))?)?;
    ledger["attempts"][0]["run_id"] = serde_json::json!("run-1002");
    ledger["attempts"][0]["tool"] = serde_json::json!("wrong-tool");
    fs::write(&ledger_path, serde_json::to_vec(&ledger)?)?;
    let conflicting = QualificationRegistryDerivationRequest {
        repository_root: root.clone(),
        current_runs_root: root.join("containers/runs"),
        attempt_ledger_path: ledger_path,
        supplemental_history_path: root.join("containers/contracts/qualification-run-history.toml"),
        existing_registry_path: None,
    };
    assert!(derive_qualification_run_registry(&conflicting).is_err());

    let runs = directory.path().join("runs");
    let failed = runs.join("run-9999-adapterremoval");
    fs::create_dir_all(&failed)?;
    fs::write(
        failed.join("evidence.json"),
        br#"{"schema_version":"bijux.container.evidence.v2","run_id":"run-9999","status":"failed"}"#,
    )?;
    let nonpassing = QualificationRegistryDerivationRequest {
        repository_root: root.clone(),
        current_runs_root: runs,
        attempt_ledger_path: root.join("containers/runs/attempt-ledger.json"),
        supplemental_history_path: root.join("containers/contracts/qualification-run-history.toml"),
        existing_registry_path: None,
    };
    assert!(derive_qualification_run_registry(&nonpassing).is_err());

    let untrusted_history = directory.path().join("qualification-run-history.toml");
    fs::write(
        &untrusted_history,
        r#"schema_version = "bijux.container.qualification_run_history.v2"
[[identities]]
run_id = "run-1126"
tool_id = "multiqc"
terminal_state = "submission_failed"
[[identities.sources]]
uri = "file:///untrusted/submission-failure.json"
sha256 = "0ac7162d4d73f760ca005701a05e96583b1a778a15e9a475d2c9b9340a9f776e"
"#,
    )?;
    let untrusted = QualificationRegistryDerivationRequest {
        repository_root: root.clone(),
        current_runs_root: root.join("containers/runs"),
        attempt_ledger_path: root.join("containers/runs/attempt-ledger.json"),
        supplemental_history_path: untrusted_history.clone(),
        existing_registry_path: None,
    };
    assert!(derive_qualification_run_registry(&untrusted).is_err());

    fs::write(
        &untrusted_history,
        r#"schema_version = "bijux.container.qualification_run_history.v2"
identities = []
[[tombstones]]
run_id = "run-1081"
disposition = "reusable"
reason = "invalid disposition"
"#,
    )?;
    let malformed_tombstone = QualificationRegistryDerivationRequest {
        repository_root: root.clone(),
        current_runs_root: root.join("containers/runs"),
        attempt_ledger_path: root.join("containers/runs/attempt-ledger.json"),
        supplemental_history_path: untrusted_history,
        existing_registry_path: None,
    };
    assert!(derive_qualification_run_registry(&malformed_tombstone).is_err());
    Ok(())
}

#[test]
fn checked_in_reservations_survive_out_of_order_attempts() -> Result<()> {
    let root = repository_root()?;
    let test_artifacts = root.join("artifacts/tests/qualification-run-registry");
    fs::create_dir_all(&test_artifacts)?;
    let directory = tempfile::tempdir_in(test_artifacts)?;
    let ledger_path = directory.path().join("attempt-ledger.json");
    let mut ledger: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("containers/runs/attempt-ledger.json"))?)?;
    ledger["attempts"]
        .as_array_mut()
        .context("attempt ledger rows")?
        .push(serde_json::json!({"run_id": "run-1157", "tool": "yleaf"}));
    fs::write(&ledger_path, serde_json::to_vec(&ledger)?)?;

    let derived = derive_qualification_run_registry(&QualificationRegistryDerivationRequest {
        repository_root: root.clone(),
        current_runs_root: root.join("containers/runs"),
        attempt_ledger_path: ledger_path,
        supplemental_history_path: root.join("containers/contracts/qualification-run-history.toml"),
        existing_registry_path: Some(root.join("containers/contracts/qualification-runs.toml")),
    })?;

    assert_eq!(derived.audit.historical_max_run_number, 1157);
    assert_eq!(run_id(&derived, "bijux_dna"), Some("run-1141"));
    assert_eq!(run_id(&derived, "fastq_screen"), Some("run-1142"));
    assert_eq!(run_id(&derived, "yleaf"), Some("run-1157"));
    Ok(())
}

#[test]
fn derivation_refreshes_stale_catalog_identity_without_reallocating_runs() -> Result<()> {
    let root = repository_root()?;
    let directory = tempfile::tempdir()?;
    let registry_path = directory.path().join("qualification-runs.toml");
    let current = derive_qualification_run_registry(&request(&root))?;
    let mut stale = current.registry.clone();
    stale.catalog_sha256 = "0".repeat(64);
    stale
        .tools
        .iter_mut()
        .find(|row| row.tool_id == "adapterremoval")
        .context("adapterremoval row")?
        .stage_id = "fastq.trim_reads".to_string();
    fs::write(&registry_path, render_qualification_run_registry(&stale)?)?;
    assert!(load_qualification_run_registry(&root, &registry_path).is_err());

    let refreshed = derive_qualification_run_registry(&QualificationRegistryDerivationRequest {
        repository_root: root.clone(),
        current_runs_root: root.join("containers/runs"),
        attempt_ledger_path: root.join("containers/runs/attempt-ledger.json"),
        supplemental_history_path: root.join("containers/contracts/qualification-run-history.toml"),
        existing_registry_path: Some(registry_path),
    })?;

    assert_eq!(refreshed.registry.catalog_sha256, current.registry.catalog_sha256);
    assert_eq!(run_id(&refreshed, "adapterremoval"), Some("run-1002"));
    assert_eq!(refreshed.registry.tools[0].stage_id, current.registry.tools[0].stage_id);
    assert_eq!(run_id(&refreshed, "bijux_dna"), Some("run-1141"));
    assert_eq!(run_id(&refreshed, "yleaf"), Some("run-1157"));
    Ok(())
}

#[test]
fn tombstones_cannot_be_reused_by_checked_in_reservations() -> Result<()> {
    let root = repository_root()?;
    let directory = tempfile::tempdir()?;
    let history_path = directory.path().join("qualification-run-history.toml");
    fs::write(
        &history_path,
        r#"schema_version = "bijux.container.qualification_run_history.v2"
identities = []
[[tombstones]]
run_id = "run-1141"
disposition = "reserved_never_reuse"
reason = "exercise collision rejection"
"#,
    )?;
    let request = QualificationRegistryDerivationRequest {
        repository_root: root.clone(),
        current_runs_root: root.join("containers/runs"),
        attempt_ledger_path: root.join("containers/runs/attempt-ledger.json"),
        supplemental_history_path: history_path,
        existing_registry_path: Some(root.join("containers/contracts/qualification-runs.toml")),
    };

    assert!(derive_qualification_run_registry(&request).is_err());
    Ok(())
}

fn request(root: &std::path::Path) -> QualificationRegistryDerivationRequest {
    QualificationRegistryDerivationRequest {
        repository_root: root.to_path_buf(),
        current_runs_root: root.join("containers/runs"),
        attempt_ledger_path: root.join("containers/runs/attempt-ledger.json"),
        supplemental_history_path: root.join("containers/contracts/qualification-run-history.toml"),
        existing_registry_path: None,
    }
}

fn run_id<'a>(
    derived: &'a bijux_dna_containers::api::DerivedQualificationRunRegistry,
    tool_id: &str,
) -> Option<&'a str> {
    derived
        .registry
        .tools
        .iter()
        .find(|row| row.tool_id == tool_id)
        .and_then(|row| row.run_id.as_deref())
}

fn repository_root() -> Result<std::path::PathBuf> {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(std::path::Path::to_path_buf)
        .context("resolve repository root")
}
