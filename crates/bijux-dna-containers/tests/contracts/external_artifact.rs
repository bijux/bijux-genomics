use std::collections::{BTreeMap, BTreeSet};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Result;
use bijux_dna_containers::api::{
    admit_external_artifact, bind_external_artifact_command, load_external_artifact_admission,
    repository_head_commit, verify_external_artifact_admission, ExternalArtifactAdmissionRequest,
    ExternalArtifactAdmissionStatus, ExternalArtifactInput, ExternalArtifactKind,
};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::{
    canonical_artifact_contract_catalog, canonical_external_artifact_tool_bindings,
    ToolArtifactSourceV1,
};

#[test]
fn external_artifact_catalog_and_stage_bindings_have_exact_pairwise_closure() -> Result<()> {
    let catalog = bijux_dna_containers::api::load_container_catalog(repository_root())?;
    let catalog_pairs = catalog
        .tools
        .iter()
        .filter(|tool| tool.external_artifact_supported)
        .flat_map(|tool| tool.stage_ids.iter().map(|stage_id| (stage_id.clone(), tool.id.clone())))
        .collect::<BTreeSet<_>>();
    let catalog_tools = catalog
        .tools
        .iter()
        .filter(|tool| tool.external_artifact_supported)
        .map(|tool| tool.id.clone())
        .collect::<BTreeSet<_>>();

    let bindings = canonical_external_artifact_tool_bindings()?;
    let binding_pairs = bindings
        .bindings()
        .map(|binding| (binding.stage_id.to_string(), binding.tool_id.to_string()))
        .collect::<BTreeSet<_>>();
    let binding_tools =
        bindings.bindings().map(|binding| binding.tool_id.to_string()).collect::<BTreeSet<_>>();

    assert_eq!(
        catalog_tools,
        BTreeSet::from_iter(["angsd", "fastuniq", "impute5", "ngsbriggs"].map(str::to_string))
    );
    assert_eq!(binding_tools, catalog_tools);
    assert_eq!(binding_pairs, catalog_pairs);
    assert_eq!(binding_pairs.len(), 8);

    let artifact_contracts = canonical_artifact_contract_catalog()
        .contracts
        .into_iter()
        .map(|contract| contract.contract_id)
        .collect::<BTreeSet<_>>();
    for binding in bindings.bindings() {
        assert!(matches!(
            &binding.artifact_source,
            ToolArtifactSourceV1::ExternalArtifact {
                checksum_required: true,
                version_probe,
                ..
            } if version_probe.iter().any(|token| token.contains("${executable}"))
        ));
        for contract_id in binding
            .inputs
            .iter()
            .map(|input| &input.artifact_contract_id)
            .chain(binding.outputs.iter().map(|output| &output.artifact_contract_id))
        {
            assert!(
                artifact_contracts.contains(contract_id),
                "binding {} references unknown artifact contract {contract_id}",
                binding.binding_id
            );
        }
    }
    Ok(())
}

#[test]
fn operator_binary_is_admitted_bound_and_reverified_without_copying() -> Result<()> {
    let root = repository_root();
    let temporary = tempfile::tempdir()?;
    let binary = temporary.path().join("angsd");
    let active = temporary.path().join("active");
    std::fs::create_dir(&active)?;
    std::fs::write(&binary, b"#!/bin/sh\nprintf 'angsd version 0.941\\n'\n")?;
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))?;
    let canonical_binary = binary.canonicalize()?;
    let checksum = bijux_dna_infra::hash_file_sha256(&binary)?;
    let record_path = temporary.path().join("angsd-admission.json");
    let commit = repository_head_commit(root)?;

    let record = admit_external_artifact(
        &ExternalArtifactAdmissionRequest {
            repository_root: root,
            distributable_image_root: &active,
            repository_commit: &commit,
            tool_id: "angsd",
            recorded_at_utc: "2026-08-06T12:00:00Z",
            artifact: Some(ExternalArtifactInput {
                kind: ExternalArtifactKind::Binary,
                path: &binary,
                expected_sha256: &checksum,
                lawful_access_basis: "institutional operator installation",
                apptainer_executable: None,
            }),
            blocker_reason: None,
        },
        &record_path,
    )?;
    assert_eq!(record.status, ExternalArtifactAdmissionStatus::Admitted);
    assert_eq!(
        record.artifact.as_ref().map(|artifact| artifact.path.as_str()),
        canonical_binary.to_str()
    );
    assert!(record_path.is_file());

    let binding = bind_external_artifact_command(
        root,
        &active,
        &record_path,
        "bam.sex",
        &["angsd".to_string(), "-bam".to_string(), "input.list".to_string()],
        None,
    )?;
    assert_eq!(binding.executable, canonical_binary.display().to_string());
    assert_eq!(binding.arguments, ["-bam", "input.list"]);

    let shell_binding = bind_external_artifact_command(
        root,
        &active,
        &record_path,
        "bam.sex",
        &["/bin/sh".to_string(), "-c".to_string(), "angsd -i sample.bam -doCounts 1".to_string()],
        None,
    )?;
    assert_eq!(shell_binding.executable, "/bin/sh");
    assert_eq!(shell_binding.arguments[0], "-c");
    assert!(shell_binding.arguments[1]
        .starts_with(&format!("'{}' -i sample.bam", canonical_binary.display())));
    assert!(bind_external_artifact_command(
        root,
        &active,
        &record_path,
        "bam.sex",
        &["/bin/sh".to_string(), "-c".to_string(), "printf '%s' angsd".to_string()],
        None,
    )
    .is_err());
    verify_external_artifact_admission(root, &active, &record_path)?;

    std::fs::write(&binary, b"#!/bin/sh\nprintf 'changed\\n'\n")?;
    assert!(verify_external_artifact_admission(root, &active, &record_path).is_err());

    let active_binary = active.join("angsd");
    std::fs::write(&active_binary, b"#!/bin/sh\nprintf 'angsd version 0.941\\n'\n")?;
    std::fs::set_permissions(&active_binary, std::fs::Permissions::from_mode(0o755))?;
    let active_checksum = bijux_dna_infra::hash_file_sha256(&active_binary)?;
    assert!(admit_external_artifact(
        &ExternalArtifactAdmissionRequest {
            repository_root: root,
            distributable_image_root: &active,
            repository_commit: &commit,
            tool_id: "angsd",
            recorded_at_utc: "2026-08-06T12:00:00Z",
            artifact: Some(ExternalArtifactInput {
                kind: ExternalArtifactKind::Binary,
                path: &active_binary,
                expected_sha256: &active_checksum,
                lawful_access_basis: "institutional operator installation",
                apptainer_executable: None,
            }),
            blocker_reason: None,
        },
        &temporary.path().join("active-root-admission.json"),
    )
    .is_err());
    Ok(())
}

#[test]
fn every_license_restricted_tool_has_an_explicit_blocked_access_record() -> Result<()> {
    let root = repository_root();
    let temporary = tempfile::tempdir()?;
    let active = temporary.path().join("active");
    std::fs::create_dir(&active)?;
    let commit = repository_head_commit(root)?;
    let bindings = canonical_external_artifact_tool_bindings()?;
    let mut blocked_stages = BTreeMap::new();
    for tool in ["angsd", "fastuniq", "impute5", "ngsbriggs"] {
        let path = temporary.path().join(format!("{tool}.json"));
        let record = admit_external_artifact(
            &ExternalArtifactAdmissionRequest {
                repository_root: root,
                distributable_image_root: &active,
                repository_commit: &commit,
                tool_id: tool,
                recorded_at_utc: "2026-08-06T12:00:00Z",
                artifact: None,
                blocker_reason: Some("no licensed institutional artifact was supplied"),
            },
            &path,
        )?;
        assert_eq!(record.status, ExternalArtifactAdmissionStatus::BlockedExternalAccess);
        assert!(!record.redistribution_permitted);
        for stage_id in &record.stage_ids {
            assert!(
                bindings.binding(&StageId::new(stage_id), &ToolId::new(tool)).is_some(),
                "blocked access must preserve the {stage_id}/{tool} product binding"
            );
        }
        blocked_stages.insert(tool, record.stage_ids.clone());
        assert!(bind_external_artifact_command(
            root,
            &active,
            &path,
            &record.stage_ids[0],
            std::slice::from_ref(&record.expected_binary),
            None,
        )
        .is_err());
        assert_eq!(load_external_artifact_admission(&path)?, record);
    }
    let blocked_pairs = blocked_stages
        .iter()
        .flat_map(|(tool_id, stage_ids)| {
            stage_ids.iter().map(|stage_id| (stage_id.clone(), (*tool_id).to_string()))
        })
        .collect::<BTreeSet<_>>();
    let binding_pairs = bindings
        .bindings()
        .map(|binding| (binding.stage_id.to_string(), binding.tool_id.to_string()))
        .collect::<BTreeSet<_>>();
    assert_eq!(blocked_pairs, binding_pairs);
    Ok(())
}

fn repository_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("container crate must live under repository crates/"))
}
