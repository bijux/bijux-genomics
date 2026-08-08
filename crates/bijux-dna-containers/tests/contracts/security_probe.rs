use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use bijux_dna_containers::api::{
    admit_security_probe_publication, SecurityProbeArtifact, SecurityProbeAuthority,
    SecurityProbePublication, SecurityProbeStatus, ToolSecurityProbe,
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
        structural_report: Some(artifact.clone()),
        runtime_report: Some(artifact.clone()),
        raw_manifest: artifact.clone(),
        error: None,
    };
    let publication = SecurityProbePublication {
        schema_version: SECURITY_PROBE_PUBLICATION_SCHEMA_V1.to_string(),
        repository_commit: authority.repository_commit.clone(),
        catalog_sha256: authority.catalog_sha256.clone(),
        inventory_sha256: authority.inventory_sha256.clone(),
        scanner_completion_sha256: authority.scanner_completion_sha256.clone(),
        database_completion_sha256: authority.database_completion_sha256.clone(),
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
