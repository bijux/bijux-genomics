use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use bijux_dna_containers::api as containers;
use bijux_dna_policies::{
    classify_vulnerability_database, derive_security_evidence, CollectedSecurityToolObservation,
    EvidenceVerdict, ImageSignatureDisposition, RuntimeProbeObservation, ScannerReportObservation,
    SecurityArtifactIdentity, SecurityEvidenceDerivationRequest, SecuritySourceIdentity,
    SecurityToolObservation, StructuralProbeObservation, VulnerabilityScannerIdentity,
};

use super::{path_from_arg, success_line, ContainerCommandOutcome, Result, Workspace};

const USAGE: &str = "Usage:\n  bijux-dna-dev containers run security-evidence -- <authority-repository-root> <study-root> <inventory-publication> <raw-publication> <scanner-contract> <scanner-evidence-root> <database-contract> <database-evidence-root> <evaluated-at-utc> <output>";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if matches!(args, [help] if matches!(help.as_str(), "--help" | "-h")) {
        return success_line(USAGE);
    }
    let [repository_root, study_root, inventory_path, raw_path, scanner_contract_path, scanner_root, database_contract_path, database_root, evaluated_at_utc, output_path] =
        args
    else {
        bail!(USAGE);
    };
    let paths = SecurityEvidencePaths {
        repository_root: path_from_arg(workspace, repository_root),
        study_root: path_from_arg(workspace, study_root),
        inventory_path: path_from_arg(workspace, inventory_path),
        raw_path: path_from_arg(workspace, raw_path),
        scanner_contract_path: path_from_arg(workspace, scanner_contract_path),
        scanner_root: path_from_arg(workspace, scanner_root),
        database_contract_path: path_from_arg(workspace, database_contract_path),
        database_root: path_from_arg(workspace, database_root),
        output_path: path_from_arg(workspace, output_path),
    };
    let publication = compose(&paths, evaluated_at_utc)?;
    persist_immutable_publication(&paths.study_root, &paths.output_path, &publication)?;
    let encoded = serde_json::to_vec_pretty(&publication)?;
    let failed =
        publication.tools.values().filter(|tool| tool.verdict == EvidenceVerdict::Failed).count();
    success_line(serde_json::to_string_pretty(&serde_json::json!({
        "failed_tools": failed,
        "output": paths.output_path,
        "publication_sha256": bijux_dna_infra::hash_bytes_sha256(&encoded),
        "tool_count": publication.tools.len(),
    }))?)
}

struct SecurityEvidencePaths {
    repository_root: PathBuf,
    study_root: PathBuf,
    inventory_path: PathBuf,
    raw_path: PathBuf,
    scanner_contract_path: PathBuf,
    scanner_root: PathBuf,
    database_contract_path: PathBuf,
    database_root: PathBuf,
    output_path: PathBuf,
}

fn compose(
    paths: &SecurityEvidencePaths,
    evaluated_at_utc: &str,
) -> Result<bijux_dna_policies::ContainerSecurityEvidencePublication> {
    let study_root = paths.study_root.canonicalize().context("canonicalize governed study root")?;
    require_under_study(&study_root, &paths.repository_root, "authority repository")?;
    require_under_study(&study_root, &paths.inventory_path, "image inventory")?;
    require_under_study(&study_root, &paths.raw_path, "raw security publication")?;
    require_under_study(&study_root, &paths.scanner_root, "security scanner evidence")?;
    require_under_study(&study_root, &paths.database_root, "vulnerability database evidence")?;

    let repository_commit = containers::clean_repository_head_commit(&paths.repository_root)?;
    let catalog_sha256 =
        containers::catalog_identity_at_commit(&paths.repository_root, &repository_commit)?;
    let catalog = containers::load_container_catalog(&paths.repository_root)?;
    let inventory = containers::load_image_inventory_publication(&paths.inventory_path)?;
    containers::validate_container_inventory_catalog(&catalog_sha256, &catalog, &inventory)?;
    if inventory.repository_commit != repository_commit {
        bail!("security inventory repository commit differs from the authority checkout");
    }
    let inventory_sha256 = bijux_dna_infra::hash_file_sha256(&paths.inventory_path)?;

    let scanner = containers::verify_security_scanner_toolchain(
        &paths.scanner_contract_path,
        &paths.scanner_root,
    )?;
    let scanner_completion_sha256 =
        bijux_dna_infra::hash_file_sha256(&paths.scanner_root.join("completion.json"))?;
    let database_contract =
        containers::load_grype_database_contract(&paths.database_contract_path)?;
    let database = containers::verify_grype_database(
        &paths.database_contract_path,
        &paths.scanner_root,
        &paths.database_root,
    )?;
    let database_completion_sha256 =
        bijux_dna_infra::hash_file_sha256(&paths.database_root.join("completion.json"))?;
    let authority = containers::SecurityProbeAuthority {
        repository_commit: repository_commit.clone(),
        catalog_sha256: catalog_sha256.clone(),
        inventory_sha256: inventory_sha256.clone(),
        scanner_completion_sha256,
        database_completion_sha256,
    };
    let admitted =
        containers::admit_security_probe_publication(&paths.raw_path, &study_root, &authority)?;

    let inventory_by_tool = inventory
        .tools
        .iter()
        .filter(|tool| {
            tool.support_disposition == containers::ContainerSupportDisposition::Retained
        })
        .map(|tool| (tool.tool.as_str(), tool))
        .collect::<BTreeMap<_, _>>();
    if inventory_by_tool.len() != admitted.publication.tools.len()
        || admitted
            .publication
            .tools
            .keys()
            .any(|tool_id| !inventory_by_tool.contains_key(tool_id.as_str()))
    {
        bail!("raw security tool set differs from the retained image inventory");
    }

    let mut observations = BTreeMap::new();
    for (tool_id, raw) in &admitted.publication.tools {
        if raw.status != containers::SecurityProbeStatus::Collected {
            bail!("retained tool {tool_id} lacks collected raw security evidence");
        }
        let inventory_tool = inventory_by_tool
            .get(tool_id.as_str())
            .with_context(|| format!("inventory omits retained tool {tool_id}"))?;
        if inventory_tool.state != containers::ContainerImageState::Current
            || inventory_tool.selected_image_sha256.as_deref() != Some(&raw.image_sha256)
        {
            bail!("raw security image differs from the current inventory for {tool_id}");
        }
        let catalog_tool = catalog
            .tools
            .iter()
            .find(|tool| tool.id == *tool_id)
            .with_context(|| format!("catalog omits retained tool {tool_id}"))?;
        let contract_commit = containers::resolve_current_apptainer_contract_commit(
            &paths.repository_root,
            &repository_commit,
            &inventory,
            catalog_tool,
        )?;
        let contract = containers::derive_apptainer_tool_contract(
            &paths.repository_root,
            tool_id,
            &contract_commit,
            &catalog.apptainer_architecture,
        )?;
        let contract_sha256 = bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&contract)?);
        if raw.version != contract.version
            || raw.tool_contract_sha256 != contract_sha256
            || raw.build_evidence_id != contract.build_evidence_id
        {
            bail!("raw security tool contract differs from authority for {tool_id}");
        }
        let reports = admitted
            .reports
            .get(tool_id)
            .with_context(|| format!("admission omits reports for {tool_id}"))?;
        let structural = reports
            .structural
            .clone()
            .with_context(|| format!("admission omits structural report for {tool_id}"))?;
        let world_writable_paths = policy_world_writable_paths(&structural);
        let runtime = reports
            .runtime
            .as_ref()
            .with_context(|| format!("admission omits runtime report for {tool_id}"))?;
        let non_root_stdout = admitted.read_artifact(&runtime.non_root_uid.stdout)?;
        let observed_uid = parse_observed_uid(&non_root_stdout)?;
        let signature_stdout =
            required_artifact(raw.signature_stdout.as_ref(), tool_id, "signature stdout")?;
        let signature_stderr =
            required_artifact(raw.signature_stderr.as_ref(), tool_id, "signature stderr")?;
        let signature_stderr_bytes = admitted.read_artifact(signature_stderr)?;
        let signature = classify_signature(raw.signature_verified, &signature_stderr_bytes);
        let signature_evidence_sha256 =
            bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&[
                signature_stdout,
                signature_stderr,
            ])?);
        let vulnerability_report =
            required_artifact(raw.vulnerability_report.as_ref(), tool_id, "vulnerability report")?;
        let secret_report =
            required_artifact(raw.secret_report.as_ref(), tool_id, "secret report")?;
        let observation = CollectedSecurityToolObservation {
            tool_id: tool_id.clone(),
            version: raw.version.clone(),
            image_sha256: raw.image_sha256.clone(),
            tool_contract_sha256: raw.tool_contract_sha256.clone(),
            build_evidence_id: raw.build_evidence_id.clone(),
            source_identity: source_identity(&contract)?,
            structural: StructuralProbeObservation {
                inspection_sha256: required_artifact(
                    raw.structural_report.as_ref(),
                    tool_id,
                    "structural report",
                )?
                .sha256
                .clone(),
                signature_evidence_sha256,
                signature,
                suid_sgid_paths: structural.suid_sgid_paths,
                world_writable_paths,
            },
            sbom_sha256: required_artifact(raw.sbom.as_ref(), tool_id, "SBOM")?.sha256.clone(),
            vulnerability_report: ScannerReportObservation {
                sha256: vulnerability_report.sha256.clone(),
                raw_json: admitted.read_artifact(vulnerability_report)?,
            },
            secret_report: ScannerReportObservation {
                sha256: secret_report.sha256.clone(),
                raw_json: admitted.read_artifact(secret_report)?,
            },
            runtime: RuntimeProbeObservation {
                probe_sha256: required_artifact(
                    raw.runtime_report.as_ref(),
                    tool_id,
                    "runtime report",
                )?
                .sha256
                .clone(),
                network_isolated: runtime.network_isolated.exit_code == 0,
                system_read_only: runtime.system_read_only.exit_code == 0,
                non_root_probe_passed: runtime.non_root_uid.exit_code == 0,
                observed_uid: Some(observed_uid),
            },
            waivers: Vec::new(),
        };
        observations
            .insert(tool_id.clone(), SecurityToolObservation::Collected(Box::new(observation)));
    }

    let grype = scanner.binaries.get("grype").context("scanner admission omits Grype")?;
    let syft = scanner.binaries.get("syft").context("scanner admission omits Syft")?;
    let gitleaks = scanner.binaries.get("gitleaks").context("scanner admission omits Gitleaks")?;
    let database_identity = classify_vulnerability_database(
        evaluated_at_utc,
        &database.status.built,
        database.database_sha256,
        database_contract.maximum_age_days,
    )?;
    derive_security_evidence(SecurityEvidenceDerivationRequest {
        evaluated_at_utc: evaluated_at_utc.to_string(),
        repository_commit,
        catalog_sha256,
        inventory_sha256,
        raw_evidence: SecurityArtifactIdentity {
            uri: admitted.publication_uri,
            sha256: admitted.publication_sha256,
        },
        scanner: VulnerabilityScannerIdentity {
            name: "grype".to_string(),
            version: grype.version.clone(),
            database: database_identity,
        },
        sbom_generator_version: syft.version.clone(),
        secret_scanner_version: gitleaks.version.clone(),
        tools: observations,
    })
}

fn required_artifact<'a>(
    artifact: Option<&'a containers::SecurityProbeArtifact>,
    tool_id: &str,
    label: &str,
) -> Result<&'a containers::SecurityProbeArtifact> {
    artifact.with_context(|| format!("collected security evidence omits {label} for {tool_id}"))
}

fn source_identity(contract: &containers::ContainerToolContract) -> Result<SecuritySourceIdentity> {
    match (&contract.source_sha256, &contract.pinned_commit) {
        (Some(sha256), None) => Ok(SecuritySourceIdentity::SourceSha256(sha256.clone())),
        (None, Some(commit)) => Ok(SecuritySourceIdentity::PinnedCommit(commit.clone())),
        _ => bail!("container tool contract has ambiguous source identity"),
    }
}

fn classify_signature(verified: bool, stderr: &[u8]) -> ImageSignatureDisposition {
    if verified {
        return ImageSignatureDisposition::SignedVerified;
    }
    let stderr = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    if stderr.contains("signature not found") {
        ImageSignatureDisposition::MissingPolicy
    } else {
        ImageSignatureDisposition::Invalid
    }
}

fn policy_world_writable_paths(report: &containers::SecurityStructuralReport) -> Vec<String> {
    if report.world_writable_observations.is_empty() {
        return report.world_writable_paths.clone();
    }
    report
        .world_writable_observations
        .iter()
        .filter(|observation| {
            observation.kind != containers::SecurityFilesystemKind::Directory
                || observation.mode & 0o1000 == 0
        })
        .map(|observation| observation.path.clone())
        .collect()
}

fn parse_observed_uid(raw: &[u8]) -> Result<u32> {
    let value = std::str::from_utf8(raw)?.trim();
    if value.is_empty() || value.lines().count() != 1 {
        bail!("non-root security probe emitted an invalid UID observation");
    }
    value.parse().context("parse non-root security probe UID")
}

fn require_under_study(study_root: &Path, path: &Path, label: &str) -> Result<()> {
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if !canonical.starts_with(study_root) {
        bail!("{label} is outside the governed study root");
    }
    Ok(())
}

fn persist_immutable_publication<T: serde::Serialize>(
    study_root: &Path,
    output: &Path,
    publication: &T,
) -> Result<()> {
    let study_root = study_root.canonicalize()?;
    let parent = output.parent().context("security evidence output has no parent")?;
    let parent = parent.canonicalize().context("canonicalize security evidence output parent")?;
    if !parent.starts_with(&study_root) {
        bail!("security evidence output is outside the governed study root");
    }
    let expected = serde_json::to_value(publication)?;
    if output.exists() {
        if !fs::symlink_metadata(output)?.file_type().is_file() {
            bail!("security evidence output must be a regular file");
        }
        let existing: serde_json::Value = serde_json::from_slice(&fs::read(output)?)?;
        if existing == expected {
            return Ok(());
        }
        bail!("immutable security evidence conflicts at {}", output.display());
    }
    bijux_dna_infra::atomic_create_json(output, publication)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_requires_exact_composition_inputs() {
        let workspace = Workspace { root: "/workspace".into() };
        assert!(run(&workspace, &[]).is_err());
        assert!(run(&workspace, &["too-few".into()]).is_err());
        assert!(run(&workspace, &["--help".into()]).is_ok());
    }

    #[test]
    fn signature_failure_classification_distinguishes_missing_and_invalid() {
        assert_eq!(
            classify_signature(false, b"integrity: signature not found for object group 1"),
            ImageSignatureDisposition::MissingPolicy
        );
        assert_eq!(
            classify_signature(false, b"signature verification failed"),
            ImageSignatureDisposition::Invalid
        );
        assert_eq!(classify_signature(true, b""), ImageSignatureDisposition::SignedVerified);
    }

    #[test]
    fn uid_parser_requires_one_numeric_observation() -> Result<()> {
        assert_eq!(parse_observed_uid(b"1000\n")?, 1000);
        assert!(parse_observed_uid(b"").is_err());
        assert!(parse_observed_uid(b"1000\n1001\n").is_err());
        Ok(())
    }

    #[test]
    fn policy_excludes_only_observed_sticky_directories() {
        let report = containers::SecurityStructuralReport {
            suid_sgid_paths: Vec::new(),
            world_writable_paths: vec!["/tmp".to_string(), "/unsafe".to_string()],
            suid_sgid_observations: Vec::new(),
            world_writable_observations: vec![
                containers::SecurityFilesystemObservation {
                    path: "/tmp".to_string(),
                    mode: 0o1777,
                    kind: containers::SecurityFilesystemKind::Directory,
                },
                containers::SecurityFilesystemObservation {
                    path: "/unsafe".to_string(),
                    mode: 0o777,
                    kind: containers::SecurityFilesystemKind::Directory,
                },
            ],
        };
        assert_eq!(policy_world_writable_paths(&report), ["/unsafe"]);

        let legacy = containers::SecurityStructuralReport {
            world_writable_observations: Vec::new(),
            ..report
        };
        assert_eq!(policy_world_writable_paths(&legacy), ["/tmp", "/unsafe"]);
    }
}
