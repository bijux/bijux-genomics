//! Thin composition of container-owned image selection and runner-owned execution bindings.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{ContainerCatalog, CurrentApptainerImage};
use bijux_dna_runner::command_runner::{
    BoundFileIdentityV1, CommandExecutionBindingV1, ExecutionBindingManifestV1,
    RUNNER_EXECUTION_BINDINGS_SCHEMA_V1,
};

use super::{path_from_arg, success_line, ContainerCommandOutcome, Workspace};

const USAGE: &str = "Usage: cargo run -p bijux-dna-dev -- containers run generate-execution-bindings -- <repository-root> <study-root> <image-inventory> <apptainer-executable> <output> <tool-id>...";

struct GenerateRequest {
    repository_root: PathBuf,
    study_root: PathBuf,
    inventory_path: PathBuf,
    apptainer_executable: PathBuf,
    output_path: PathBuf,
    tool_ids: Vec<String>,
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
    )?;
    bijux_dna_infra::atomic_create_json(&request.output_path, &manifest)?;
    success_line(format!(
        "generated {} verified runner execution bindings at {}",
        manifest.bindings.len(),
        request.output_path.display()
    ))
}

fn parse_request(workspace: &Workspace, args: &[String]) -> Result<GenerateRequest> {
    let [repository_root, study_root, inventory_path, apptainer_executable, output_path, tool_ids @ ..] =
        args
    else {
        bail!(USAGE);
    };
    if tool_ids.is_empty() {
        bail!(USAGE);
    }
    let mut unique = BTreeSet::new();
    for tool_id in tool_ids {
        require_identifier(tool_id, "tool id")?;
        if !unique.insert(tool_id.clone()) {
            bail!("execution binding request repeats tool id {tool_id}");
        }
    }
    Ok(GenerateRequest {
        repository_root: path_from_arg(workspace, repository_root),
        study_root: path_from_arg(workspace, study_root),
        inventory_path: path_from_arg(workspace, inventory_path),
        apptainer_executable: path_from_arg(workspace, apptainer_executable),
        output_path: path_from_arg(workspace, output_path),
        tool_ids: unique.into_iter().collect(),
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
) -> Result<ExecutionBindingManifestV1> {
    let mut commands = BTreeSet::new();
    let mut bindings = Vec::with_capacity(tool_ids.len());
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
    bindings.sort_by(|left, right| left.command.cmp(&right.command));
    Ok(ExecutionBindingManifestV1 {
        schema_version: RUNNER_EXECUTION_BINDINGS_SCHEMA_V1.to_string(),
        bindings,
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
        let mut duplicate = args.to_vec();
        duplicate.push("bcftools".to_string());
        let Err(error) = parse_request(&workspace, &duplicate) else {
            bail!("duplicate tool id was accepted");
        };
        assert!(error.to_string().contains("repeats tool id bcftools"));
        Ok(())
    }
}
