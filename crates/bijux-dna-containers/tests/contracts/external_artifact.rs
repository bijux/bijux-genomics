use std::collections::{BTreeMap, BTreeSet};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_containers::api::{
    admit_external_artifact, bind_external_artifact_command, load_external_artifact_admission,
    repository_head_commit, verify_external_artifact_admission, ContainerArtifactDelivery,
    ContainerProductSupport, ExternalArtifactAdmissionRequest, ExternalArtifactAdmissionStatus,
    ExternalArtifactInput, ExternalArtifactKind, EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2,
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
fn historical_v1_blocker_migrates_without_rewriting_immutable_evidence() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let path = temporary.path().join("angsd-v1.json");
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.container.external_artifact_admission.v1",
            "status": "blocked_external_access",
            "recorded_at_utc": "2026-08-09T22:44:40Z",
            "repository_commit": "e250bead897f2138424906cb22f034fee47e3804",
            "catalog": {
                "path": "containers/contracts/catalog.toml",
                "sha256": "0".repeat(64),
            },
            "distributable_image_root": "/governed/active-images",
            "tool_id": "angsd",
            "version": "0.941",
            "support_disposition": "unsupported",
            "support_reason": "license terms prohibit repository redistribution",
            "stage_ids": ["bam.kinship", "bam.sex", "vcf.call_gl", "vcf.call_pseudohaploid"],
            "expected_binary": "angsd",
            "version_command": "angsd --version",
            "expected_version_regex": "v?[0-9]+[.][0-9]+([.-][0-9A-Za-z]+)?",
            "redistribution_permitted": false,
            "blocker_reason": "no operator-provided checksum-bound artifact was available",
        }))?,
    )?;

    let migrated = load_external_artifact_admission(&path)?;
    assert_eq!(migrated.schema_version, EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2);
    assert_eq!(migrated.availability, ExternalArtifactAdmissionStatus::BlockedExternalAccess);
    assert_eq!(migrated.product_support, ContainerProductSupport::Supported);
    assert_eq!(migrated.artifact_delivery, ContainerArtifactDelivery::ExternalArtifact);
    assert!(!migrated.redistribution_permitted);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(&path)?)?["schema_version"],
        "bijux.container.external_artifact_admission.v1"
    );
    Ok(())
}

#[test]
fn operator_binary_is_admitted_bound_and_reverified_without_copying() -> Result<()> {
    let (temporary, repository) = external_artifact_fixture()?;
    let root = repository.as_path();
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
            availability: ExternalArtifactAdmissionStatus::Admitted,
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
    assert_eq!(record.availability, ExternalArtifactAdmissionStatus::Admitted);
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
    verify_repository_location_rejection(root, &active, &binary, temporary.path(), &commit)?;
    verify_rebound_artifact_rejection(root, &active, &record_path, &binary, temporary.path())?;
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
            availability: ExternalArtifactAdmissionStatus::Admitted,
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
                availability: ExternalArtifactAdmissionStatus::BlockedExternalAccess,
                artifact: None,
                blocker_reason: Some("no licensed institutional artifact was supplied"),
            },
            &path,
        )?;
        assert_eq!(record.availability, ExternalArtifactAdmissionStatus::BlockedExternalAccess);
        assert_eq!(record.product_support, ContainerProductSupport::Supported);
        assert_eq!(record.artifact_delivery, ContainerArtifactDelivery::ExternalArtifact);
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

#[test]
fn unavailable_artifact_is_distinct_from_blocked_access() -> Result<()> {
    let root = repository_root();
    let temporary = tempfile::tempdir()?;
    let active = temporary.path().join("active");
    std::fs::create_dir(&active)?;
    let commit = repository_head_commit(root)?;
    let path = temporary.path().join("impute5-unavailable.json");
    let record = admit_external_artifact(
        &ExternalArtifactAdmissionRequest {
            repository_root: root,
            distributable_image_root: &active,
            repository_commit: &commit,
            tool_id: "impute5",
            recorded_at_utc: "2026-08-16T06:45:00Z",
            availability: ExternalArtifactAdmissionStatus::Unavailable,
            artifact: None,
            blocker_reason: Some("no artifact exists on the audited site surfaces"),
        },
        &path,
    )?;
    assert_eq!(record.availability, ExternalArtifactAdmissionStatus::Unavailable);
    assert_eq!(record.product_support, ContainerProductSupport::Supported);
    let error = bind_external_artifact_command(
        root,
        &active,
        &path,
        "vcf.impute",
        &["impute5".to_string(), "--help".to_string()],
        None,
    )
    .err()
    .context("unavailable artifact must fail before command binding")?;
    assert!(error.to_string().contains("external artifact unavailable for impute5"));
    Ok(())
}

fn repository_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("container crate must live under repository crates/"))
}

// Admission requires a committed catalog. Keep the operator installation beside the
// private product repository so its location is independent of the caller's TMPDIR.
fn external_artifact_fixture() -> Result<(tempfile::TempDir, PathBuf)> {
    let owner = bijux_dna_testkit::tempdir_for("container-external-artifact");
    let repository = owner.path().join("repository");
    std::fs::create_dir_all(repository.join("containers/contracts"))?;
    std::fs::copy(
        repository_root().join("containers/contracts/catalog.toml"),
        repository.join("containers/contracts/catalog.toml"),
    )?;
    for arguments in [
        &["init", "--quiet"][..],
        &["add", "containers/contracts/catalog.toml"][..],
        &["commit", "--quiet", "--no-gpg-sign", "-m", "test: record external artifact catalog"][..],
    ] {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args([
                "-c",
                "user.name=Bijux Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(arguments)
            .output()?;
        anyhow::ensure!(
            output.status.success(),
            "prepare committed external-artifact fixture: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok((owner, repository.canonicalize()?))
}

fn verify_repository_location_rejection(
    root: &Path,
    active: &Path,
    binary: &Path,
    fixture_root: &Path,
    commit: &str,
) -> Result<()> {
    let internal = root.join("operator-angsd");
    std::fs::copy(binary, &internal)?;
    let checksum = bijux_dna_infra::hash_file_sha256(&internal)?;
    let output = fixture_root.join("repository-root-admission.json");
    let result = admit_external_artifact(
        &ExternalArtifactAdmissionRequest {
            repository_root: root,
            distributable_image_root: active,
            repository_commit: commit,
            tool_id: "angsd",
            recorded_at_utc: "2026-08-06T12:00:00Z",
            availability: ExternalArtifactAdmissionStatus::Admitted,
            artifact: Some(ExternalArtifactInput {
                kind: ExternalArtifactKind::Binary,
                path: &internal,
                expected_sha256: &checksum,
                lawful_access_basis: "local contract fixture",
                apptainer_executable: None,
            }),
            blocker_reason: None,
        },
        &output,
    );
    assert!(result.is_err_and(|error| {
        error.to_string().contains("external artifact must remain outside the product repository")
    }));
    assert!(!output.exists());
    Ok(())
}

fn verify_rebound_artifact_rejection(
    root: &Path,
    active: &Path,
    record_path: &Path,
    binary: &Path,
    fixture_root: &Path,
) -> Result<()> {
    let admitted_bytes = std::fs::read(record_path)?;
    let mut relative: serde_json::Value = serde_json::from_slice(&admitted_bytes)?;
    relative["artifact"]["path"] = serde_json::json!("relative-angsd");
    relative["version_probe"]["command"][0] = serde_json::json!("relative-angsd");
    std::fs::write(record_path, serde_json::to_vec(&relative)?)?;
    assert!(load_external_artifact_admission(record_path).is_err());
    std::fs::write(record_path, &admitted_bytes)?;

    let held_binary = fixture_root.join("held-angsd");
    std::fs::rename(binary, &held_binary)?;
    let linked_binary = active.join("same-byte-angsd");
    std::fs::copy(&held_binary, &linked_binary)?;
    for target in [&held_binary, &linked_binary] {
        std::os::unix::fs::symlink(target, binary)?;
        assert!(verify_external_artifact_admission(root, active, record_path).is_err());
        assert!(bind_external_artifact_command(
            root,
            active,
            record_path,
            "bam.sex",
            &["angsd".into()],
            None
        )
        .is_err());
        assert_eq!(std::fs::read(record_path)?, admitted_bytes);
        std::fs::remove_file(binary)?;
    }
    std::fs::rename(&held_binary, binary)?;
    verify_external_artifact_admission(root, active, record_path)?;

    Ok(())
}

#[test]
fn excessive_external_probe_output_rejects_before_admission_publication() -> Result<()> {
    let (temporary, repository) = external_artifact_fixture()?;
    let root = repository.as_path();
    let binary = temporary.path().join("angsd");
    let active = temporary.path().join("active");
    std::fs::create_dir(&active)?;
    std::fs::write(&binary, b"#!/bin/sh\nwhile :; do printf 01234567890123456789; done\n")?;
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))?;
    let checksum = bijux_dna_infra::hash_file_sha256(&binary)?;
    let record_path = temporary.path().join("admission.json");
    let commit = repository_head_commit(root)?;
    let started = std::time::Instant::now();
    let result = admit_external_artifact(
        &ExternalArtifactAdmissionRequest {
            repository_root: root,
            distributable_image_root: &active,
            repository_commit: &commit,
            tool_id: "angsd",
            recorded_at_utc: "2026-08-06T12:00:00Z",
            availability: ExternalArtifactAdmissionStatus::Admitted,
            artifact: Some(ExternalArtifactInput {
                kind: ExternalArtifactKind::Binary,
                path: &binary,
                expected_sha256: &checksum,
                lawful_access_basis: "local contract fixture",
                apptainer_executable: None,
            }),
            blocker_reason: None,
        },
        &record_path,
    );
    assert!(result.is_err_and(|error| error.to_string().contains("bounded evidence limit")));
    assert!(!record_path.exists());
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
    Ok(())
}

mod version_transport {
    use super::*;
    use bijux_dna_containers::api::{
        admit_external_artifact_with_probe, ExternalArtifactProbeObservation,
        ExternalArtifactProbeOutcome, ExternalArtifactProbeRequest, ExternalArtifactProbeTransport,
    };
    struct Observer {
        calls: Vec<Vec<String>>,
        next: Option<Result<ExternalArtifactProbeObservation>>,
    }
    impl ExternalArtifactProbeTransport for Observer {
        fn observe(
            &mut self,
            request: &ExternalArtifactProbeRequest<'_>,
        ) -> Result<ExternalArtifactProbeObservation> {
            assert_eq!(request.timeout, std::time::Duration::from_secs(30));
            assert_eq!(request.max_output_bytes, 64 * 1024);
            self.calls.push(request.command.to_vec());
            self.next.take().context("unexpected version observation")?
        }
    }
    fn observation(
        outcome: ExternalArtifactProbeOutcome,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    ) -> ExternalArtifactProbeObservation {
        ExternalArtifactProbeObservation { outcome, stdout, stderr }
    }
    fn reject_unusable_evidence(
        request: &ExternalArtifactAdmissionRequest<'_>,
        temporary: &tempfile::TempDir,
        observer: &mut Observer,
    ) {
        let success = ExternalArtifactProbeOutcome::Exited { code: Some(0) };
        let invalid = [
            Ok(observation(
                ExternalArtifactProbeOutcome::TimedOut,
                b"angsd 0.941".to_vec(),
                vec![],
            )),
            Ok(observation(
                ExternalArtifactProbeOutcome::OutputLimitExceeded,
                b"angsd 0.941".to_vec(),
                vec![],
            )),
            Ok(observation(
                ExternalArtifactProbeOutcome::Exited { code: Some(7) },
                b"angsd 0.941".to_vec(),
                vec![],
            )),
            Ok(observation(
                ExternalArtifactProbeOutcome::Exited { code: None },
                b"angsd 0.941".to_vec(),
                vec![],
            )),
            Ok(observation(success, vec![255], vec![])),
            Ok(observation(success, b"unknown version".to_vec(), vec![])),
            Ok(observation(success, vec![], vec![])),
            Ok(observation(success, vec![b'a'; 32 * 1024], vec![b'b'; 32 * 1024 + 1])),
            Err(anyhow::anyhow!("version transport unavailable")),
        ];
        for (index, evidence) in invalid.into_iter().enumerate() {
            observer.next = Some(evidence);
            let output = temporary.path().join(format!("rejected-{index}.json"));
            assert!(admit_external_artifact_with_probe(request, &output, observer).is_err());
            assert!(!output.exists());
        }
    }

    #[test]
    fn preserves_projection_and_rejects_unusable_evidence() -> Result<()> {
        let success = ExternalArtifactProbeOutcome::Exited { code: Some(0) };
        let (temporary, repository) = external_artifact_fixture()?;
        let root = repository.as_path();
        let artifact = temporary.path().join("operator artifact with spaces");
        std::fs::write(
            &artifact,
            b"opaque operator bytes; injected transport must not execute this",
        )?;
        let active = temporary.path().join("active");
        std::fs::create_dir(&active)?;
        let checksum = bijux_dna_infra::hash_file_sha256(&artifact)?;
        let commit = repository_head_commit(root)?;
        let mut request = ExternalArtifactAdmissionRequest {
            repository_root: root,
            distributable_image_root: &active,
            repository_commit: &commit,
            tool_id: "angsd",
            recorded_at_utc: "2026-08-06T12:00:00Z",
            availability: ExternalArtifactAdmissionStatus::Admitted,
            artifact: Some(ExternalArtifactInput {
                kind: ExternalArtifactKind::Binary,
                path: &artifact,
                expected_sha256: &checksum,
                lawful_access_basis: "local contract fixture",
                apptainer_executable: None,
            }),
            blocker_reason: None,
        };
        let mut observer = Observer { calls: Vec::new(), next: None };
        reject_unusable_evidence(&request, &temporary, &mut observer);
        let calls = observer.calls.len();
        request.artifact.as_mut().context("artifact input")?.expected_sha256 =
            "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(admit_external_artifact_with_probe(
            &request,
            &temporary.path().join("bad-checksum.json"),
            &mut observer
        )
        .is_err());
        assert_eq!(observer.calls.len(), calls);
        request.artifact.as_mut().context("artifact input")?.expected_sha256 = &checksum;
        for kind in [ExternalArtifactKind::Binary, ExternalArtifactKind::ApptainerSif] {
            let input = request.artifact.as_mut().context("artifact input")?;
            input.kind = kind;
            input.apptainer_executable = Some(Path::new("/operator runtime/apptainer"));
            observer.next =
                Some(Ok(observation(success, b"angsd".to_vec(), b"version 0.941\n".to_vec())));
            let output = temporary.path().join(format!("admitted-{}.json", kind.as_str()));
            let record = admit_external_artifact_with_probe(&request, &output, &mut observer)?;
            let path =
                artifact.canonicalize()?.to_str().context("artifact UTF-8 path")?.to_string();
            let expected = match kind {
                ExternalArtifactKind::Binary => vec![path, "--version".to_string()],
                ExternalArtifactKind::ApptainerSif => vec![
                    "/operator runtime/apptainer".to_string(),
                    "exec".to_string(),
                    path,
                    "angsd".to_string(),
                    "--version".to_string(),
                ],
            };
            assert_eq!(observer.calls.last(), Some(&expected));
            let probe = record.version_probe.as_ref().context("admitted version probe")?;
            assert_eq!(probe.command, expected);
            assert_eq!(probe.output, "angsd\nversion 0.941\n");
            assert_eq!(
                probe.output_sha256,
                bijux_dna_infra::hash_bytes_sha256(probe.output.as_bytes())
            );
            assert_eq!(
                serde_json::to_value(load_external_artifact_admission(&output)?)?,
                serde_json::to_value(record)?
            );
        }
        let calls = observer.calls.len();
        request.artifact = None;
        request.blocker_reason = Some("operator artifact is unavailable");
        for availability in [
            ExternalArtifactAdmissionStatus::Unavailable,
            ExternalArtifactAdmissionStatus::BlockedExternalAccess,
        ] {
            request.availability = availability;
            let output = temporary.path().join(format!("availability-{availability:?}.json"));
            let record = admit_external_artifact_with_probe(&request, &output, &mut observer)?;
            assert_eq!(record.availability, availability);
            assert!(record.version_probe.is_none());
            assert!(record.artifact.is_none());
        }
        assert_eq!(observer.calls.len(), calls);
        Ok(())
    }
}

#[path = "external_artifact/observation.rs"]
mod observation;
