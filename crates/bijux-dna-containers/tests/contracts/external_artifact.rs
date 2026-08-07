use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Result;
use bijux_dna_containers::api::{
    admit_external_artifact, bind_external_artifact_command, load_external_artifact_admission,
    repository_head_commit, verify_external_artifact_admission, ExternalArtifactAdmissionRequest,
    ExternalArtifactAdmissionStatus, ExternalArtifactInput, ExternalArtifactKind,
};

#[test]
fn operator_binary_is_admitted_bound_and_reverified_without_copying() -> Result<()> {
    let root = repository_root();
    let temporary = tempfile::tempdir()?;
    let binary = temporary.path().join("angsd");
    let active = temporary.path().join("active");
    std::fs::create_dir(&active)?;
    std::fs::write(&binary, b"#!/bin/sh\nprintf 'angsd version 0.941\\n'\n")?;
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))?;
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
    assert_eq!(record.artifact.as_ref().map(|artifact| artifact.path.as_str()), binary.to_str());
    assert!(record_path.is_file());

    let binding = bind_external_artifact_command(
        root,
        &active,
        &record_path,
        "bam.sex",
        &["angsd".to_string(), "-bam".to_string(), "input.list".to_string()],
        None,
    )?;
    assert_eq!(binding.executable, binary.display().to_string());
    assert_eq!(binding.arguments, ["-bam", "input.list"]);
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
    Ok(())
}

fn repository_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("container crate must live under repository crates/"))
}
