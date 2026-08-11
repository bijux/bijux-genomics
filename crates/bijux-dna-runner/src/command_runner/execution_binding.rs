use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

pub const RUNNER_EXECUTION_BINDINGS_ENV: &str = "BIJUX_DNA_RUNNER_BINDINGS";
pub const RUNNER_EXECUTION_BINDINGS_SCHEMA_V1: &str = "bijux.runner.execution_bindings.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundFileIdentityV1 {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandExecutionBindingV1 {
    pub command: String,
    pub program: String,
    #[serde(default)]
    pub prefix_args: Vec<String>,
    #[serde(default)]
    pub required_files: Vec<BoundFileIdentityV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionBindingManifestV1 {
    pub schema_version: String,
    pub bindings: Vec<CommandExecutionBindingV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedCommand {
    pub program: String,
    pub args: Vec<String>,
}

pub(super) fn resolve_command(
    command: &str,
    args: &[String],
    envs: Option<&BTreeMap<String, String>>,
) -> Result<ResolvedCommand> {
    if command.trim().is_empty() {
        bail!("command executable must not be empty");
    }
    require_nonempty(command, "command executable")?;
    let Some(path) = configured_manifest_path(envs)? else {
        return Ok(ResolvedCommand { program: command.to_string(), args: args.to_vec() });
    };
    let manifest = load_manifest(&path)?;
    let Some(binding) = manifest.bindings.iter().find(|binding| binding.command == command) else {
        return Ok(ResolvedCommand { program: command.to_string(), args: args.to_vec() });
    };
    verify_required_files(binding)?;
    let mut bound_args = binding.prefix_args.clone();
    bound_args.extend_from_slice(args);
    Ok(ResolvedCommand { program: binding.program.clone(), args: bound_args })
}

fn configured_manifest_path(envs: Option<&BTreeMap<String, String>>) -> Result<Option<PathBuf>> {
    let configured = envs
        .and_then(|values| values.get(RUNNER_EXECUTION_BINDINGS_ENV).cloned())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os(RUNNER_EXECUTION_BINDINGS_ENV).map(PathBuf::from));
    let Some(path) = configured else {
        return Ok(None);
    };
    if !path.is_absolute() {
        bail!("{RUNNER_EXECUTION_BINDINGS_ENV} must name an absolute manifest path");
    }
    Ok(Some(path))
}

fn load_manifest(path: &Path) -> Result<ExecutionBindingManifestV1> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect runner execution binding manifest {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("runner execution binding manifest must be a regular non-symlink file");
    }
    let bytes = std::fs::read(path)
        .with_context(|| format!("read runner execution binding manifest {}", path.display()))?;
    let manifest: ExecutionBindingManifestV1 = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse runner execution binding manifest {}", path.display()))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_manifest(manifest: &ExecutionBindingManifestV1) -> Result<()> {
    if manifest.schema_version != RUNNER_EXECUTION_BINDINGS_SCHEMA_V1 {
        bail!(
            "unsupported runner execution binding schema {}; expected {RUNNER_EXECUTION_BINDINGS_SCHEMA_V1}",
            manifest.schema_version
        );
    }
    if manifest.bindings.is_empty() {
        bail!("runner execution binding manifest requires at least one binding");
    }
    let mut commands = BTreeSet::new();
    let mut previous_command: Option<&str> = None;
    for binding in &manifest.bindings {
        require_nonempty(&binding.command, "bound command")?;
        require_nonempty(&binding.program, "bound program")?;
        if !commands.insert(binding.command.as_str()) {
            bail!("runner execution binding repeats command {}", binding.command);
        }
        if previous_command.is_some_and(|previous| previous >= binding.command.as_str()) {
            bail!("runner execution bindings must be sorted by command");
        }
        previous_command = Some(binding.command.as_str());
        validate_required_files(binding)?;
    }
    Ok(())
}

fn validate_required_files(binding: &CommandExecutionBindingV1) -> Result<()> {
    let mut paths = BTreeSet::new();
    let mut previous_path: Option<&str> = None;
    for identity in &binding.required_files {
        require_nonempty(&identity.path, "bound file path")?;
        let path = Path::new(&identity.path);
        if !path.is_absolute() {
            bail!("bound file path must be absolute: {}", identity.path);
        }
        validate_sha256(&identity.sha256)?;
        if !paths.insert(identity.path.as_str()) {
            bail!("runner execution binding repeats required file {}", identity.path);
        }
        if previous_path.is_some_and(|previous| previous >= identity.path.as_str()) {
            bail!("runner execution binding required files must be sorted by path");
        }
        previous_path = Some(identity.path.as_str());
        if binding.program != identity.path
            && !binding.prefix_args.iter().any(|argument| argument == &identity.path)
        {
            bail!(
                "required file {} is not part of the bound invocation for {}",
                identity.path,
                binding.command
            );
        }
    }
    Ok(())
}

fn verify_required_files(binding: &CommandExecutionBindingV1) -> Result<()> {
    for identity in &binding.required_files {
        let path = Path::new(&identity.path);
        let metadata = std::fs::symlink_metadata(path)
            .with_context(|| format!("inspect bound file {}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("bound file must be a regular non-symlink file: {}", path.display());
        }
        if metadata.len() != identity.size_bytes {
            bail!("bound file size differs from declared identity: {}", path.display());
        }
        let observed = bijux_dna_infra::hash_file_sha256(path)
            .with_context(|| format!("hash bound file {}", path.display()))?;
        if observed != identity.sha256 {
            bail!("bound file checksum differs from declared identity: {}", path.display());
        }
    }
    Ok(())
}

fn require_nonempty(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() || value.trim() != value {
        bail!("{label} must be non-empty without surrounding whitespace");
    }
    Ok(())
}

fn validate_sha256(value: &str) -> Result<()> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value != value.to_ascii_lowercase()
    {
        bail!("bound file checksum must be lowercase SHA-256");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(path: &Path, sha256: String, size_bytes: u64) -> ExecutionBindingManifestV1 {
        ExecutionBindingManifestV1 {
            schema_version: RUNNER_EXECUTION_BINDINGS_SCHEMA_V1.to_string(),
            bindings: vec![CommandExecutionBindingV1 {
                command: "bcftools".to_string(),
                program: "/usr/bin/apptainer".to_string(),
                prefix_args: vec![
                    "exec".to_string(),
                    "--cleanenv".to_string(),
                    path.display().to_string(),
                    "bcftools".to_string(),
                ],
                required_files: vec![BoundFileIdentityV1 {
                    path: path.display().to_string(),
                    sha256,
                    size_bytes,
                }],
            }],
        }
    }

    fn write_manifest(root: &Path, manifest: &ExecutionBindingManifestV1) -> Result<PathBuf> {
        let path = root.join("execution-bindings.json");
        std::fs::write(&path, serde_json::to_vec_pretty(manifest)?)?;
        Ok(path)
    }

    #[test]
    fn binding_resolves_exact_program_prefix_and_requested_args() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let artifact = temporary.path().join("bcftools.sif");
        std::fs::write(&artifact, b"admitted-image")?;
        let sha256 = bijux_dna_infra::hash_file_sha256(&artifact)?;
        let manifest = binding(&artifact, sha256, artifact.metadata()?.len());
        let manifest_path = write_manifest(temporary.path(), &manifest)?;
        let envs = BTreeMap::from([(
            RUNNER_EXECUTION_BINDINGS_ENV.to_string(),
            manifest_path.display().to_string(),
        )]);

        let resolved = resolve_command(
            "bcftools",
            &["view".to_string(), "sample.vcf.gz".to_string()],
            Some(&envs),
        )?;

        assert_eq!(resolved.program, "/usr/bin/apptainer");
        assert_eq!(
            resolved.args,
            vec![
                "exec",
                "--cleanenv",
                artifact.to_str().unwrap_or_default(),
                "bcftools",
                "view",
                "sample.vcf.gz"
            ]
        );
        Ok(())
    }

    #[test]
    fn binding_rejects_artifact_identity_drift_before_spawn() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let artifact = temporary.path().join("bcftools.sif");
        std::fs::write(&artifact, b"admitted-image")?;
        let manifest = binding(&artifact, "0".repeat(64), artifact.metadata()?.len());
        let manifest_path = write_manifest(temporary.path(), &manifest)?;
        let envs = BTreeMap::from([(
            RUNNER_EXECUTION_BINDINGS_ENV.to_string(),
            manifest_path.display().to_string(),
        )]);

        let Err(error) = resolve_command("bcftools", &[], Some(&envs)) else {
            anyhow::bail!("checksum drift was accepted");
        };

        assert!(error.to_string().contains("checksum differs"));
        Ok(())
    }

    #[test]
    fn binding_manifest_rejects_duplicate_commands() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let artifact = temporary.path().join("bcftools.sif");
        std::fs::write(&artifact, b"admitted-image")?;
        let sha256 = bijux_dna_infra::hash_file_sha256(&artifact)?;
        let mut manifest = binding(&artifact, sha256, artifact.metadata()?.len());
        manifest.bindings.push(manifest.bindings[0].clone());
        let manifest_path = write_manifest(temporary.path(), &manifest)?;
        let envs = BTreeMap::from([(
            RUNNER_EXECUTION_BINDINGS_ENV.to_string(),
            manifest_path.display().to_string(),
        )]);

        let Err(error) = resolve_command("bcftools", &[], Some(&envs)) else {
            anyhow::bail!("duplicate binding was accepted");
        };

        assert!(error.to_string().contains("repeats command"));
        Ok(())
    }

    #[test]
    fn unbound_commands_preserve_the_requested_invocation() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let artifact = temporary.path().join("bcftools.sif");
        std::fs::write(&artifact, b"admitted-image")?;
        let sha256 = bijux_dna_infra::hash_file_sha256(&artifact)?;
        let manifest = binding(&artifact, sha256, artifact.metadata()?.len());
        let manifest_path = write_manifest(temporary.path(), &manifest)?;
        let envs = BTreeMap::from([(
            RUNNER_EXECUTION_BINDINGS_ENV.to_string(),
            manifest_path.display().to_string(),
        )]);

        let resolved = resolve_command("git", &["status".to_string()], Some(&envs))?;

        assert_eq!(resolved.program, "git");
        assert_eq!(resolved.args, ["status"]);
        Ok(())
    }
}
