//! Thin composition of container-owned image selection and runner-owned execution bindings.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{
    ContainerCatalog, CurrentApptainerImage, ExternalArtifactAdmission,
    ExternalArtifactAdmissionStatus, ExternalArtifactKind,
};
use bijux_dna_runner::command_runner::{
    BoundFileIdentityV1, CommandExecutionBindingV1, ExecutionBindingManifestV1,
    RUNNER_EXECUTION_BINDINGS_SCHEMA_V1,
};

use super::{path_from_arg, success_line, ContainerCommandOutcome, Workspace};

const USAGE: &str = "Usage: cargo run -p bijux-dna-dev -- containers run generate-execution-bindings -- <repository-root> <study-root> <image-inventory> <apptainer-executable> <output> (<tool-id> | --external-admission <distributable-image-root> <record>)...";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ExternalAdmissionSelection {
    distributable_image_root: PathBuf,
    record_path: PathBuf,
}

struct GenerateRequest {
    repository_root: PathBuf,
    study_root: PathBuf,
    inventory_path: PathBuf,
    apptainer_executable: PathBuf,
    output_path: PathBuf,
    tool_ids: Vec<String>,
    external_admissions: Vec<ExternalAdmissionSelection>,
}

pub(super) fn generate(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(USAGE);
    }
    let request = parse_request(workspace, args)?;
    if request.output_path.try_exists()? {
        bail!("refusing to replace existing runner execution binding manifest");
    }
    validate_apptainer_executable(&request.apptainer_executable)?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&request.repository_root)?;
    let catalog = bijux_dna_containers::api::load_container_catalog(&request.repository_root)?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &request.repository_root.join("containers/contracts/catalog.toml"),
    )?;
    let inventory =
        bijux_dna_containers::api::load_image_inventory_publication(&request.inventory_path)?;
    let manifest = derive_manifest(
        &catalog,
        &catalog_sha256,
        &inventory,
        &repository_commit,
        &request.study_root,
        &request.apptainer_executable,
        &request.tool_ids,
        &request.external_admissions,
        &request.repository_root,
    )?;
    bijux_dna_infra::atomic_create_json(&request.output_path, &manifest)?;
    success_line(format!(
        "generated {} verified runner execution bindings at {}",
        manifest.bindings.len(),
        request.output_path.display()
    ))
}

fn parse_request(workspace: &Workspace, args: &[String]) -> Result<GenerateRequest> {
    let [repository_root, study_root, inventory_path, apptainer_executable, output_path, selections @ ..] =
        args
    else {
        bail!(USAGE);
    };
    if selections.is_empty() {
        bail!(USAGE);
    }
    let mut tool_ids = BTreeSet::new();
    let mut external_admissions = BTreeSet::new();
    let mut selections = selections.iter();
    while let Some(selection) = selections.next() {
        if selection == "--external-admission" {
            let root = selections
                .next()
                .context("--external-admission requires a distributable image root")?;
            let record =
                selections.next().context("--external-admission requires a record path")?;
            let selection = ExternalAdmissionSelection {
                distributable_image_root: path_from_arg(workspace, root),
                record_path: path_from_arg(workspace, record),
            };
            if !external_admissions.insert(selection.clone()) {
                bail!(
                    "execution binding request repeats external admission {}",
                    selection.record_path.display()
                );
            }
        } else {
            require_identifier(selection, "tool id")?;
            if !tool_ids.insert(selection.clone()) {
                bail!("execution binding request repeats tool id {selection}");
            }
        }
    }
    Ok(GenerateRequest {
        repository_root: path_from_arg(workspace, repository_root),
        study_root: path_from_arg(workspace, study_root),
        inventory_path: path_from_arg(workspace, inventory_path),
        apptainer_executable: path_from_arg(workspace, apptainer_executable),
        output_path: path_from_arg(workspace, output_path),
        tool_ids: tool_ids.into_iter().collect(),
        external_admissions: external_admissions.into_iter().collect(),
    })
}

fn derive_manifest(
    catalog: &ContainerCatalog,
    catalog_sha256: &str,
    inventory: &bijux_dna_containers::api::ContainerImageInventoryPublication,
    repository_commit: &str,
    study_root: &Path,
    apptainer_executable: &Path,
    tool_ids: &[String],
    external_admissions: &[ExternalAdmissionSelection],
    repository_root: &Path,
) -> Result<ExecutionBindingManifestV1> {
    let mut commands = BTreeSet::new();
    let mut bindings = Vec::with_capacity(tool_ids.len() + external_admissions.len());
    for tool_id in tool_ids {
        let tool = catalog
            .tools
            .iter()
            .find(|tool| tool.id == *tool_id)
            .with_context(|| format!("container catalog does not declare tool {tool_id}"))?;
        let image = bijux_dna_containers::api::resolve_catalog_locked_apptainer_image(
            inventory,
            catalog_sha256,
            catalog,
            repository_commit,
            study_root,
            tool_id,
        )?;
        require_identifier(&tool.expected_bin, "expected binary")?;
        if !commands.insert(tool.expected_bin.clone()) {
            bail!("selected tools resolve the duplicate executable command {}", tool.expected_bin);
        }
        bindings.push(binding_from_image(&image, &tool.expected_bin, apptainer_executable));
    }
    for selection in external_admissions {
        let admission = bijux_dna_containers::api::verify_external_artifact_admission(
            repository_root,
            &selection.distributable_image_root,
            &selection.record_path,
        )?;
        require_identifier(&admission.expected_binary, "expected binary")?;
        if !commands.insert(admission.expected_binary.clone()) {
            bail!(
                "selected artifacts resolve the duplicate executable command {}",
                admission.expected_binary
            );
        }
        bindings.push(binding_from_external_artifact(&admission, apptainer_executable)?);
    }
    bindings.sort_by(|left, right| left.command.cmp(&right.command));
    Ok(ExecutionBindingManifestV1 {
        schema_version: RUNNER_EXECUTION_BINDINGS_SCHEMA_V1.to_string(),
        bindings,
    })
}

fn binding_from_external_artifact(
    admission: &ExternalArtifactAdmission,
    apptainer_executable: &Path,
) -> Result<CommandExecutionBindingV1> {
    if admission.availability != ExternalArtifactAdmissionStatus::Admitted {
        bail!(
            "external artifact {} is not admitted: {:?}",
            admission.tool_id,
            admission.availability
        );
    }
    let artifact = admission
        .artifact
        .as_ref()
        .context("admitted external artifact has no persisted identity")?;
    let (program, prefix_args) = match artifact.kind {
        ExternalArtifactKind::Binary => (artifact.path.clone(), Vec::new()),
        ExternalArtifactKind::ApptainerSif => (
            apptainer_executable.display().to_string(),
            vec![
                "exec".to_string(),
                "--cleanenv".to_string(),
                artifact.path.clone(),
                admission.expected_binary.clone(),
            ],
        ),
    };
    Ok(CommandExecutionBindingV1 {
        command: admission.expected_binary.clone(),
        program,
        prefix_args,
        required_files: vec![BoundFileIdentityV1 {
            path: artifact.path.clone(),
            sha256: artifact.sha256.clone(),
            size_bytes: artifact.size_bytes,
        }],
    })
}

fn binding_from_image(
    image: &CurrentApptainerImage,
    expected_binary: &str,
    apptainer_executable: &Path,
) -> CommandExecutionBindingV1 {
    let image_path = image.path.display().to_string();
    CommandExecutionBindingV1 {
        command: expected_binary.to_string(),
        program: apptainer_executable.display().to_string(),
        prefix_args: vec![
            "exec".to_string(),
            "--cleanenv".to_string(),
            image_path.clone(),
            expected_binary.to_string(),
        ],
        required_files: vec![BoundFileIdentityV1 {
            path: image_path,
            sha256: image.sha256.clone(),
            size_bytes: image.size_bytes,
        }],
    }
}

fn validate_apptainer_executable(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        bail!("Apptainer executable path must be absolute");
    }
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("inspect Apptainer executable {}", path.display()))?;
    if !metadata.is_file() {
        bail!("Apptainer executable is not a regular file: {}", path.display());
    }
    Ok(())
}

fn require_identifier(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty()
        || value.trim() != value
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        bail!("{label} is invalid: {value}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn external_admission(
        kind: ExternalArtifactKind,
        availability: ExternalArtifactAdmissionStatus,
    ) -> ExternalArtifactAdmission {
        ExternalArtifactAdmission {
            schema_version: "bijux.container.external_artifact_admission.v2".to_string(),
            availability,
            recorded_at_utc: "2026-08-11T00:00:00Z".to_string(),
            repository_commit: "a".repeat(40),
            catalog: bijux_dna_containers::api::EvidenceSourceIdentity {
                path: "containers/contracts/catalog.toml".to_string(),
                sha256: "b".repeat(64),
            },
            distributable_image_root: "/study/containers/current-images".to_string(),
            tool_id: "angsd".to_string(),
            version: "0.941".to_string(),
            product_support: bijux_dna_containers::api::ContainerProductSupport::Supported,
            artifact_delivery:
                bijux_dna_containers::api::ContainerArtifactDelivery::ExternalArtifact,
            delivery_reason: "operator-provided artifact".to_string(),
            stage_ids: vec!["bam.genotyping".to_string()],
            expected_binary: "angsd".to_string(),
            version_command: "angsd --version".to_string(),
            expected_version_regex: "ANGSD version: 0.941".to_string(),
            redistribution_permitted: false,
            lawful_access_basis: Some("institutional access".to_string()),
            artifact: Some(bijux_dna_containers::api::ExternalArtifactIdentity {
                kind,
                path: "/study/private/angsd".to_string(),
                sha256: "c".repeat(64),
                size_bytes: 42,
            }),
            version_probe: None,
            blocker_reason: None,
        }
    }

    #[test]
    fn resolved_image_projects_exact_runner_file_identity() {
        let image = CurrentApptainerImage {
            tool_id: "bcftools".to_string(),
            path: "/study/containers/current-images/bcftools.sif".into(),
            sha256: "5".repeat(64),
            size_bytes: 63_160_320,
            repository_commit: "a".repeat(40),
            catalog_sha256: "b".repeat(64),
        };

        let binding = binding_from_image(&image, "bcftools", Path::new("/usr/bin/apptainer"));

        assert_eq!(binding.command, "bcftools");
        assert_eq!(binding.program, "/usr/bin/apptainer");
        assert_eq!(
            binding.prefix_args,
            ["exec", "--cleanenv", "/study/containers/current-images/bcftools.sif", "bcftools"]
        );
        assert_eq!(binding.required_files.len(), 1);
        assert_eq!(binding.required_files[0].sha256, "5".repeat(64));
        assert_eq!(binding.required_files[0].size_bytes, 63_160_320);
    }

    #[test]
    fn request_parser_canonicalizes_tool_order_and_rejects_duplicates() -> Result<()> {
        let workspace = Workspace { root: "/workspace".into() };
        let args = [
            "/repository",
            "/study",
            "/inventory.json",
            "/usr/bin/apptainer",
            "/bindings.json",
            "samtools",
            "bcftools",
        ]
        .map(str::to_string);

        let request = parse_request(&workspace, &args)?;

        assert_eq!(request.tool_ids, ["bcftools", "samtools"]);
        assert!(request.external_admissions.is_empty());
        let mut duplicate = args.to_vec();
        duplicate.push("bcftools".to_string());
        let Err(error) = parse_request(&workspace, &duplicate) else {
            bail!("duplicate tool id was accepted");
        };
        assert!(error.to_string().contains("repeats tool id bcftools"));
        Ok(())
    }

    #[test]
    fn request_parser_keeps_external_admissions_separate_from_catalog_images() -> Result<()> {
        let workspace = Workspace { root: "/workspace".into() };
        let args = [
            "/repository",
            "/study",
            "/inventory.json",
            "/usr/bin/apptainer",
            "/bindings.json",
            "bcftools",
            "--external-admission",
            "/study/containers/current-images",
            "/study/admissions/angsd.json",
        ]
        .map(str::to_string);

        let request = parse_request(&workspace, &args)?;

        assert_eq!(request.tool_ids, ["bcftools"]);
        assert_eq!(
            request.external_admissions,
            [ExternalAdmissionSelection {
                distributable_image_root: PathBuf::from("/study/containers/current-images"),
                record_path: PathBuf::from("/study/admissions/angsd.json"),
            }]
        );
        Ok(())
    }

    #[test]
    fn admitted_external_artifacts_project_binary_and_sif_runner_bindings() -> Result<()> {
        let binary = binding_from_external_artifact(
            &external_admission(
                ExternalArtifactKind::Binary,
                ExternalArtifactAdmissionStatus::Admitted,
            ),
            Path::new("/usr/bin/apptainer"),
        )?;
        assert_eq!(binary.command, "angsd");
        assert_eq!(binary.program, "/study/private/angsd");
        assert!(binary.prefix_args.is_empty());
        assert_eq!(binary.required_files[0].sha256, "c".repeat(64));

        let sif = binding_from_external_artifact(
            &external_admission(
                ExternalArtifactKind::ApptainerSif,
                ExternalArtifactAdmissionStatus::Admitted,
            ),
            Path::new("/usr/bin/apptainer"),
        )?;
        assert_eq!(sif.program, "/usr/bin/apptainer");
        assert_eq!(sif.prefix_args, ["exec", "--cleanenv", "/study/private/angsd", "angsd"]);
        Ok(())
    }

    #[test]
    fn blocked_external_artifact_cannot_become_a_runner_binding() -> Result<()> {
        let Err(error) = binding_from_external_artifact(
            &external_admission(
                ExternalArtifactKind::Binary,
                ExternalArtifactAdmissionStatus::BlockedExternalAccess,
            ),
            Path::new("/usr/bin/apptainer"),
        ) else {
            bail!("blocked access produced a runner binding");
        };
        assert!(error.to_string().contains("is not admitted"));
        Ok(())
    }
}
