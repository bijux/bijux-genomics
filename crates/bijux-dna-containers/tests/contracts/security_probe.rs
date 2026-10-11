use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use bijux_dna_containers::api::{
    admit_security_probe_baseline, admit_security_probe_publication, SecurityFilesystemKind,
    SecurityFilesystemObservation, SecurityProbeArtifact, SecurityProbeAuthority,
    SecurityProbeBaselineAuthority, SecurityProbePublication, SecurityProbeStatus,
    SecurityRuntimeProbe, SecurityRuntimeReport, SecurityStructuralReport, ToolSecurityProbe,
    SECURITY_PROBE_PUBLICATION_SCHEMA_V1,
};

fn digest(value: char) -> String {
    std::iter::repeat_n(value, 64).collect()
}

fn fixture() -> Result<(tempfile::TempDir, PathBuf, SecurityProbeAuthority, SecurityProbeArtifact)>
{
    let study = bijux_dna_testkit::tempdir_for("security-probe-admission");
    let artifact_path = study.path().join("evidence/artifact.json");
    fs::create_dir_all(artifact_path.parent().context("artifact parent")?)?;
    fs::write(&artifact_path, b"{}")?;
    let artifact = SecurityProbeArtifact {
        uri: "study://evidence/artifact.json".to_string(),
        sha256: bijux_dna_infra::hash_file_sha256(&artifact_path)?,
    };
    let structural_path = study.path().join("evidence/structural.json");
    fs::write(
        &structural_path,
        serde_json::to_vec_pretty(&SecurityStructuralReport {
            suid_sgid_paths: Vec::new(),
            world_writable_paths: vec!["/tmp".to_string()],
            suid_sgid_observations: Vec::new(),
            world_writable_observations: vec![SecurityFilesystemObservation {
                path: "/tmp".to_string(),
                mode: 0o1777,
                kind: SecurityFilesystemKind::Directory,
            }],
        })?,
    )?;
    let structural = SecurityProbeArtifact {
        uri: "study://evidence/structural.json".to_string(),
        sha256: bijux_dna_infra::hash_file_sha256(&structural_path)?,
    };
    let stream_path = study.path().join("evidence/runtime-stream.txt");
    fs::write(&stream_path, b"1000\n")?;
    let stream = SecurityProbeArtifact {
        uri: "study://evidence/runtime-stream.txt".to_string(),
        sha256: bijux_dna_infra::hash_file_sha256(&stream_path)?,
    };
    let probe =
        SecurityRuntimeProbe { exit_code: 0, stdout: stream.clone(), stderr: stream.clone() };
    let runtime_path = study.path().join("evidence/runtime.json");
    fs::write(
        &runtime_path,
        serde_json::to_vec_pretty(&SecurityRuntimeReport {
            network_isolated: probe.clone(),
            non_root_uid: probe.clone(),
            system_read_only: probe,
        })?,
    )?;
    let runtime = SecurityProbeArtifact {
        uri: "study://evidence/runtime.json".to_string(),
        sha256: bijux_dna_infra::hash_file_sha256(&runtime_path)?,
    };
    let manifest_path = study.path().join("evidence/raw-manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&BTreeMap::from([
            ("artifact.json".to_string(), artifact.sha256.clone()),
            ("runtime-stream.txt".to_string(), stream.sha256),
            ("runtime.json".to_string(), runtime.sha256.clone()),
            ("structural.json".to_string(), structural.sha256.clone()),
        ]))?,
    )?;
    let raw_manifest = SecurityProbeArtifact {
        uri: "study://evidence/raw-manifest.json".to_string(),
        sha256: bijux_dna_infra::hash_file_sha256(&manifest_path)?,
    };
    let authority = SecurityProbeAuthority {
        repository_commit: "a".repeat(40),
        catalog_sha256: digest('b'),
        inventory_sha256: digest('c'),
        scanner_completion_sha256: digest('d'),
        database_completion_sha256: digest('e'),
    };
    let tool = ToolSecurityProbe {
        tool_id: "samtools".to_string(),
        version: "1.20".to_string(),
        image_sha256: digest('1'),
        tool_contract_sha256: digest('2'),
        build_evidence_id: digest('3'),
        status: SecurityProbeStatus::Collected,
        sbom: Some(artifact.clone()),
        vulnerability_report: Some(artifact.clone()),
        secret_report: Some(artifact.clone()),
        signature_stdout: Some(artifact.clone()),
        signature_stderr: Some(artifact.clone()),
        signature_verified: true,
        structural_report: Some(structural),
        runtime_report: Some(runtime),
        raw_manifest,
        error: None,
    };
    let publication = SecurityProbePublication {
        schema_version: SECURITY_PROBE_PUBLICATION_SCHEMA_V1.to_string(),
        repository_commit: authority.repository_commit.clone(),
        catalog_sha256: authority.catalog_sha256.clone(),
        inventory_sha256: authority.inventory_sha256.clone(),
        scanner_completion_sha256: authority.scanner_completion_sha256.clone(),
        database_completion_sha256: authority.database_completion_sha256.clone(),
        reused_publication_sha256: None,
        tools: BTreeMap::from([("samtools".to_string(), tool)]),
    };
    let publication_path = study.path().join("evidence/publication.json");
    fs::write(&publication_path, serde_json::to_vec_pretty(&publication)?)?;
    Ok((study, publication_path, authority, artifact))
}

#[test]
fn admits_exact_authority_and_rechecks_artifacts() -> Result<()> {
    let (study, publication_path, authority, artifact) = fixture()?;
    let admitted = admit_security_probe_publication(&publication_path, study.path(), &authority)?;
    assert_eq!(admitted.publication.tools.len(), 1);
    assert_eq!(admitted.read_artifact(&artifact)?, b"{}");
    let reports = admitted.reports.get("samtools").context("admitted samtools reports")?;
    assert!(reports.structural.is_some());
    assert_eq!(reports.runtime.as_ref().context("runtime report")?.non_root_uid.exit_code, 0);
    assert_eq!(reports.raw_manifest.len(), 4);
    Ok(())
}

#[test]
fn admits_reused_publication_lineage() -> Result<()> {
    let (study, publication_path, authority, _) = fixture()?;
    let mut publication: SecurityProbePublication =
        serde_json::from_slice(&fs::read(&publication_path)?)?;
    publication.reused_publication_sha256 = Some("c".repeat(64));
    fs::write(&publication_path, serde_json::to_vec_pretty(&publication)?)?;

    let admitted = admit_security_probe_publication(&publication_path, study.path(), &authority)?;
    assert_eq!(admitted.publication.reused_publication_sha256, Some("c".repeat(64)));
    Ok(())
}

#[test]
fn admits_baseline_across_repository_and_inventory_revisions() -> Result<()> {
    let (study, publication_path, authority, _) = fixture()?;
    let baseline_authority = SecurityProbeBaselineAuthority {
        catalog_sha256: authority.catalog_sha256,
        scanner_completion_sha256: authority.scanner_completion_sha256,
        database_completion_sha256: authority.database_completion_sha256,
    };
    let admitted =
        admit_security_probe_baseline(&publication_path, study.path(), &baseline_authority)?;
    assert_eq!(admitted.publication.repository_commit, "a".repeat(40));
    assert_eq!(admitted.publication.inventory_sha256, digest('c'));
    Ok(())
}

#[test]
fn rejects_baseline_scanner_drift() -> Result<()> {
    let (study, publication_path, authority, _) = fixture()?;
    let baseline_authority = SecurityProbeBaselineAuthority {
        catalog_sha256: authority.catalog_sha256,
        scanner_completion_sha256: digest('f'),
        database_completion_sha256: authority.database_completion_sha256,
    };
    assert!(admit_security_probe_baseline(&publication_path, study.path(), &baseline_authority)
        .is_err_and(|error| error.to_string().contains("stable authority differs")));
    Ok(())
}

#[test]
fn rejects_authority_drift() -> Result<()> {
    let (study, publication_path, mut authority, _) = fixture()?;
    authority.inventory_sha256 = digest('f');
    assert!(admit_security_probe_publication(&publication_path, study.path(), &authority)
        .is_err_and(|error| error.to_string().contains("authority differs")));
    Ok(())
}

#[test]
fn rejects_artifact_digest_drift() -> Result<()> {
    let (study, publication_path, authority, _) = fixture()?;
    fs::write(study.path().join("evidence/artifact.json"), b"changed")?;
    assert!(admit_security_probe_publication(&publication_path, study.path(), &authority)
        .is_err_and(|error| error.to_string().contains("digest differs")));
    Ok(())
}

#[test]
fn rejects_nested_runtime_stream_digest_drift() -> Result<()> {
    let (study, publication_path, authority, _) = fixture()?;
    fs::write(study.path().join("evidence/runtime-stream.txt"), b"changed")?;
    assert!(admit_security_probe_publication(&publication_path, study.path(), &authority)
        .is_err_and(|error| error.to_string().contains("digest differs")));
    Ok(())
}

#[test]
fn rejects_raw_manifest_omission() -> Result<()> {
    let (study, publication_path, authority, _) = fixture()?;
    let manifest_path = study.path().join("evidence/raw-manifest.json");
    let mut manifest: BTreeMap<String, String> =
        serde_json::from_slice(&fs::read(&manifest_path)?)?;
    manifest.remove("runtime-stream.txt");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    let mut publication: SecurityProbePublication =
        serde_json::from_slice(&fs::read(&publication_path)?)?;
    publication.tools.get_mut("samtools").context("samtools fixture")?.raw_manifest.sha256 =
        bijux_dna_infra::hash_file_sha256(&manifest_path)?;
    fs::write(&publication_path, serde_json::to_vec_pretty(&publication)?)?;
    assert!(admit_security_probe_publication(&publication_path, study.path(), &authority)
        .is_err_and(|error| error.to_string().contains("omits or changes")));
    Ok(())
}

#[test]
fn rejects_structural_observation_projection_drift() -> Result<()> {
    let (study, publication_path, authority, _) = fixture()?;
    let structural_path = study.path().join("evidence/structural.json");
    let mut structural: SecurityStructuralReport =
        serde_json::from_slice(&fs::read(&structural_path)?)?;
    structural.world_writable_observations[0].path = "/different".to_string();
    fs::write(&structural_path, serde_json::to_vec_pretty(&structural)?)?;
    let structural_sha256 = bijux_dna_infra::hash_file_sha256(&structural_path)?;

    let manifest_path = study.path().join("evidence/raw-manifest.json");
    let mut manifest: BTreeMap<String, String> =
        serde_json::from_slice(&fs::read(&manifest_path)?)?;
    manifest.insert("structural.json".to_string(), structural_sha256.clone());
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    let mut publication: SecurityProbePublication =
        serde_json::from_slice(&fs::read(&publication_path)?)?;
    let tool = publication.tools.get_mut("samtools").context("samtools fixture")?;
    tool.structural_report.as_mut().context("structural artifact")?.sha256 = structural_sha256;
    tool.raw_manifest.sha256 = bijux_dna_infra::hash_file_sha256(&manifest_path)?;
    fs::write(&publication_path, serde_json::to_vec_pretty(&publication)?)?;

    assert!(admit_security_probe_publication(&publication_path, study.path(), &authority)
        .is_err_and(|error| error.to_string().contains("path projection")));
    Ok(())
}

#[test]
fn rejects_unsafe_artifact_uri() -> Result<()> {
    let (study, publication_path, authority, _) = fixture()?;
    let mut publication: SecurityProbePublication =
        serde_json::from_slice(&fs::read(&publication_path)?)?;
    publication.tools.get_mut("samtools").context("samtools fixture")?.raw_manifest.uri =
        "study://../outside.json".to_string();
    fs::write(&publication_path, serde_json::to_vec_pretty(&publication)?)?;
    assert!(admit_security_probe_publication(&publication_path, study.path(), &authority)
        .is_err_and(|error| error.to_string().contains("unsafe path")));
    Ok(())
}

#[test]
fn rejects_incomplete_collected_state() -> Result<()> {
    let (study, publication_path, authority, _) = fixture()?;
    let mut publication: SecurityProbePublication =
        serde_json::from_slice(&fs::read(&publication_path)?)?;
    publication.tools.get_mut("samtools").context("samtools fixture")?.runtime_report = None;
    fs::write(&publication_path, serde_json::to_vec_pretty(&publication)?)?;
    assert!(admit_security_probe_publication(&publication_path, study.path(), &authority)
        .is_err_and(|error| error.to_string().contains("incomplete")));
    Ok(())
}
