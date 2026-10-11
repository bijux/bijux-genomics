use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use bijux_dna_containers::api as containers;
use bijux_dna_policies::{
    classify_vulnerability_database, derive_security_evidence, ArtifactAvailability,
    ArtifactDelivery, CollectedSecurityToolObservation, EvidenceVerdict, ImageSignatureDisposition,
    NoSecurityArtifactObservation, ProductSupport, RuntimeProbeObservation,
    ScannerReportObservation, SecurityArtifactIdentity, SecurityEvidenceDerivationRequest,
    SecuritySourceIdentity, SecurityToolObservation, StructuralProbeObservation,
    VulnerabilityScannerIdentity,
};

use super::{path_from_arg, read_utf8, success_line, ContainerCommandOutcome, Result, Workspace};

const SCHEMA_PATH: &str = "containers/contracts/schemas/security-evidence.v2.json";
const USAGE: &str = "Usage:\n  bijux-dna-dev containers run security-evidence -- compose <authority-repository-root> <study-root> <inventory-publication> <raw-publication> <scanner-contract> <scanner-evidence-root> <database-contract> <database-evidence-root> <external-artifact-admission-root> <evaluated-at-utc> <output>\n  bijux-dna-dev containers run security-evidence -- generate-schema [<output>]\n  bijux-dna-dev containers run security-evidence -- check-schema";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(USAGE);
    }
    match operation.as_str() {
        "compose" => compose_command(workspace, options),
        "generate-schema" => generate_schema(workspace, options),
        "check-schema" => check_schema(workspace, options),
        _ => bail!(USAGE),
    }
}

fn compose_command(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [repository_root, study_root, inventory_path, raw_path, scanner_contract_path, scanner_root, database_contract_path, database_root, external_artifact_admission_root, evaluated_at_utc, output_path] =
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
        external_artifact_admission_root: path_from_arg(
            workspace,
            external_artifact_admission_root,
        ),
        output_path: path_from_arg(workspace, output_path),
    };
    let publication = compose(&paths, evaluated_at_utc)?;
    persist_immutable_publication(&paths.study_root, &paths.output_path, &publication)?;
    let encoded = serde_json::to_vec_pretty(&publication)?;
    let failed =
        publication.tools.values().filter(|tool| tool.verdict == EvidenceVerdict::Failed).count();
    let blocked =
        publication.tools.values().filter(|tool| tool.verdict == EvidenceVerdict::Blocked).count();
    let missing =
        publication.tools.values().filter(|tool| tool.verdict == EvidenceVerdict::Missing).count();
    success_line(serde_json::to_string_pretty(&serde_json::json!({
        "blocked_tools": blocked,
        "failed_tools": failed,
        "missing_tools": missing,
        "output": paths.output_path,
        "publication_sha256": bijux_dna_infra::hash_bytes_sha256(&encoded),
        "tool_count": publication.tools.len(),
    }))?)
}

fn generate_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let output = match args {
        [] => workspace.path(SCHEMA_PATH),
        [output] => path_from_arg(workspace, output),
        _ => bail!(USAGE),
    };
    bijux_dna_infra::atomic_write_json(&output, &bijux_dna_policies::security_evidence_schema())?;
    success_line(format!("generated security-evidence schema at {}", output.display()))
}

fn check_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        bail!(USAGE);
    }
    let checked: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path(SCHEMA_PATH))?)?;
    let generated = serde_json::to_value(bijux_dna_policies::security_evidence_schema())?;
    if checked != generated {
        return Ok(ContainerCommandOutcome::failure(
            "security-evidence schema drifted from Rust policy types\n",
        ));
    }
    success_line("security-evidence schema: OK")
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
    external_artifact_admission_root: PathBuf,
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
    require_under_study(
        &study_root,
        &paths.external_artifact_admission_root,
        "external artifact admissions",
    )?;

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

    let observations = derive_tool_observations(
        paths,
        &study_root,
        &repository_commit,
        &catalog,
        &inventory,
        &admitted,
    )?;

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

struct CollectedToolAuthority {
    version: String,
    product_support: ProductSupport,
    artifact_delivery: ArtifactDelivery,
    redistribution_permitted: bool,
    artifact_sha256: String,
    tool_contract_sha256: String,
    admission_evidence_sha256: String,
    source_identity: SecuritySourceIdentity,
}

fn derive_tool_observations(
    paths: &SecurityEvidencePaths,
    study_root: &Path,
    repository_commit: &str,
    catalog: &containers::ContainerCatalog,
    inventory: &containers::ContainerImageInventoryPublication,
    admitted: &containers::AdmittedSecurityProbePublication,
) -> Result<BTreeMap<String, SecurityToolObservation>> {
    let inventory_by_tool =
        inventory.tools.iter().map(|tool| (tool.tool.as_str(), tool)).collect::<BTreeMap<_, _>>();
    let mut consumed_raw = BTreeSet::new();
    let mut observations = BTreeMap::new();
    for tool in &catalog.tools {
        let observation = if tool.product_support != containers::ContainerProductSupport::Supported
        {
            SecurityToolObservation::NoArtifact(NoSecurityArtifactObservation {
                tool_id: tool.id.clone(),
                version: tool.version.clone(),
                product_support: product_support(tool.product_support),
                artifact_delivery: artifact_delivery(tool.artifact_delivery),
                redistribution_permitted: tool.redistribution_permitted,
                availability: ArtifactAvailability::Unavailable,
                reason: None,
            })
        } else if tool.artifact_delivery
            == containers::ContainerArtifactDelivery::RedistributableImage
        {
            let authority = redistributable_authority(
                paths,
                repository_commit,
                catalog,
                inventory,
                inventory_by_tool.get(tool.id.as_str()).copied(),
                tool,
            )?;
            consumed_raw.insert(tool.id.clone());
            SecurityToolObservation::Collected(Box::new(collected_observation(
                admitted, tool, authority,
            )?))
        } else {
            external_artifact_observation(
                paths,
                study_root,
                repository_commit,
                admitted,
                tool,
                &mut consumed_raw,
            )?
        };
        observations.insert(tool.id.clone(), observation);
    }
    if admitted.publication.tools.keys().any(|tool_id| !consumed_raw.contains(tool_id)) {
        bail!("raw security publication contains a tool without current artifact authority");
    }
    Ok(observations)
}

fn redistributable_authority(
    paths: &SecurityEvidencePaths,
    repository_commit: &str,
    catalog: &containers::ContainerCatalog,
    inventory: &containers::ContainerImageInventoryPublication,
    inventory_tool: Option<&containers::ContainerImageInventoryTool>,
    tool: &containers::ContainerCatalogTool,
) -> Result<CollectedToolAuthority> {
    let inventory_tool = inventory_tool
        .with_context(|| format!("inventory omits redistributable tool {}", tool.id))?;
    if inventory_tool.state != containers::ContainerImageState::Current {
        bail!("redistributable tool {} lacks a current image", tool.id);
    }
    let artifact_sha256 = inventory_tool
        .selected_image_sha256
        .clone()
        .with_context(|| format!("current image inventory omits digest for {}", tool.id))?;
    let contract_commit = containers::resolve_current_apptainer_contract_commit(
        &paths.repository_root,
        repository_commit,
        inventory,
        tool,
    )?;
    let contract = containers::derive_apptainer_tool_contract(
        &paths.repository_root,
        &tool.id,
        &contract_commit,
        &catalog.apptainer_architecture,
    )?;
    Ok(CollectedToolAuthority {
        version: contract.version.clone(),
        product_support: ProductSupport::Supported,
        artifact_delivery: ArtifactDelivery::RedistributableImage,
        redistribution_permitted: true,
        artifact_sha256,
        tool_contract_sha256: bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&contract)?),
        admission_evidence_sha256: contract.build_evidence_id.clone(),
        source_identity: source_identity(&contract)?,
    })
}

fn external_artifact_observation(
    paths: &SecurityEvidencePaths,
    study_root: &Path,
    repository_commit: &str,
    admitted_security: &containers::AdmittedSecurityProbePublication,
    tool: &containers::ContainerCatalogTool,
    consumed_raw: &mut BTreeSet<String>,
) -> Result<SecurityToolObservation> {
    let path = paths.external_artifact_admission_root.join(format!("{}.json", tool.id));
    require_under_study(study_root, &path, "external artifact admission")?;
    let loaded = containers::load_external_artifact_admission(&path)?;
    let admission = containers::verify_external_artifact_admission(
        &paths.repository_root,
        Path::new(&loaded.distributable_image_root),
        &path,
    )?;
    if admission.repository_commit != repository_commit {
        bail!("external artifact admission commit differs from security authority");
    }
    let availability = artifact_availability(admission.availability);
    if availability != ArtifactAvailability::Admitted {
        if admitted_security.publication.tools.contains_key(&tool.id) {
            bail!("raw security evidence exists for unavailable external tool {}", tool.id);
        }
        return Ok(SecurityToolObservation::NoArtifact(NoSecurityArtifactObservation {
            tool_id: tool.id.clone(),
            version: tool.version.clone(),
            product_support: ProductSupport::Supported,
            artifact_delivery: artifact_delivery(tool.artifact_delivery),
            redistribution_permitted: false,
            availability,
            reason: admission.blocker_reason,
        }));
    }
    let artifact = admission.artifact.as_ref().context("admitted external tool lacks artifact")?;
    let authority = CollectedToolAuthority {
        version: admission.version.clone(),
        product_support: ProductSupport::Supported,
        artifact_delivery: artifact_delivery(admission.artifact_delivery),
        redistribution_permitted: false,
        artifact_sha256: artifact.sha256.clone(),
        tool_contract_sha256: bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(tool)?),
        admission_evidence_sha256: bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(
            &admission,
        )?),
        source_identity: SecuritySourceIdentity::SourceSha256(artifact.sha256.clone()),
    };
    consumed_raw.insert(tool.id.clone());
    Ok(SecurityToolObservation::Collected(Box::new(collected_observation(
        admitted_security,
        tool,
        authority,
    )?)))
}

fn collected_observation(
    admitted: &containers::AdmittedSecurityProbePublication,
    tool: &containers::ContainerCatalogTool,
    authority: CollectedToolAuthority,
) -> Result<CollectedSecurityToolObservation> {
    let raw =
        admitted.publication.tools.get(&tool.id).with_context(|| {
            format!("admitted artifact {} lacks raw security evidence", tool.id)
        })?;
    if raw.status != containers::SecurityProbeStatus::Collected {
        bail!("admitted artifact {} lacks collected raw security evidence", tool.id);
    }
    if raw.version != authority.version
        || raw.image_sha256 != authority.artifact_sha256
        || raw.tool_contract_sha256 != authority.tool_contract_sha256
        || raw.build_evidence_id != authority.admission_evidence_sha256
    {
        bail!("raw security artifact contract differs from authority for {}", tool.id);
    }
    let reports = admitted
        .reports
        .get(&tool.id)
        .with_context(|| format!("admission omits reports for {}", tool.id))?;
    let structural = reports
        .structural
        .clone()
        .with_context(|| format!("admission omits structural report for {}", tool.id))?;
    let runtime = reports
        .runtime
        .as_ref()
        .with_context(|| format!("admission omits runtime report for {}", tool.id))?;
    let observed_uid = parse_observed_uid(&admitted.read_artifact(&runtime.non_root_uid.stdout)?)?;
    let signature_stdout =
        required_artifact(raw.signature_stdout.as_ref(), &tool.id, "signature stdout")?;
    let signature_stderr =
        required_artifact(raw.signature_stderr.as_ref(), &tool.id, "signature stderr")?;
    let signature =
        classify_signature(raw.signature_verified, &admitted.read_artifact(signature_stderr)?);
    let signature_evidence_sha256 = bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&[
        signature_stdout,
        signature_stderr,
    ])?);
    let world_writable_paths = policy_world_writable_paths(&structural);
    let vulnerability_report =
        required_artifact(raw.vulnerability_report.as_ref(), &tool.id, "vulnerability report")?;
    let secret_report = required_artifact(raw.secret_report.as_ref(), &tool.id, "secret report")?;
    Ok(CollectedSecurityToolObservation {
        tool_id: tool.id.clone(),
        version: raw.version.clone(),
        product_support: authority.product_support,
        artifact_delivery: authority.artifact_delivery,
        redistribution_permitted: authority.redistribution_permitted,
        artifact_sha256: raw.image_sha256.clone(),
        tool_contract_sha256: raw.tool_contract_sha256.clone(),
        admission_evidence_sha256: raw.build_evidence_id.clone(),
        source_identity: authority.source_identity,
        structural: StructuralProbeObservation {
            inspection_sha256: required_artifact(
                raw.structural_report.as_ref(),
                &tool.id,
                "structural report",
            )?
            .sha256
            .clone(),
            signature_evidence_sha256,
            signature,
            suid_sgid_paths: structural.suid_sgid_paths,
            world_writable_paths,
        },
        sbom_sha256: required_artifact(raw.sbom.as_ref(), &tool.id, "SBOM")?.sha256.clone(),
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
                &tool.id,
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
    })
}

const fn product_support(value: containers::ContainerProductSupport) -> ProductSupport {
    match value {
        containers::ContainerProductSupport::Supported => ProductSupport::Supported,
        containers::ContainerProductSupport::Retired => ProductSupport::Retired,
        containers::ContainerProductSupport::Excluded => ProductSupport::Excluded,
    }
}

const fn artifact_delivery(value: containers::ContainerArtifactDelivery) -> ArtifactDelivery {
    match value {
        containers::ContainerArtifactDelivery::RedistributableImage => {
            ArtifactDelivery::RedistributableImage
        }
        containers::ContainerArtifactDelivery::ExternalArtifact => {
            ArtifactDelivery::ExternalArtifact
        }
        containers::ContainerArtifactDelivery::PrivateSiteBuild => {
            ArtifactDelivery::PrivateSiteBuild
        }
        containers::ContainerArtifactDelivery::SiteModule => ArtifactDelivery::SiteModule,
    }
}

const fn artifact_availability(
    value: containers::ExternalArtifactAdmissionStatus,
) -> ArtifactAvailability {
    match value {
        containers::ExternalArtifactAdmissionStatus::Admitted => ArtifactAvailability::Admitted,
        containers::ExternalArtifactAdmissionStatus::BlockedExternalAccess => {
            ArtifactAvailability::BlockedExternalAccess
        }
        containers::ExternalArtifactAdmissionStatus::Unavailable => {
            ArtifactAvailability::Unavailable
        }
    }
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
        assert!(run(&workspace, &["compose".into()]).is_err());
    }

    #[test]
    fn external_artifact_states_preserve_product_support() {
        assert_eq!(
            artifact_availability(
                containers::ExternalArtifactAdmissionStatus::BlockedExternalAccess
            ),
            ArtifactAvailability::BlockedExternalAccess
        );
        assert_eq!(
            artifact_delivery(containers::ContainerArtifactDelivery::ExternalArtifact),
            ArtifactDelivery::ExternalArtifact
        );
        assert_eq!(
            product_support(containers::ContainerProductSupport::Supported),
            ProductSupport::Supported
        );
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
