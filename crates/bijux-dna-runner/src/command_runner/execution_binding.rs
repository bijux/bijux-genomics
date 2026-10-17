use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

pub const RUNNER_EXECUTION_BINDINGS_ENV: &str = "BIJUX_DNA_RUNNER_BINDINGS";
pub const RUNNER_EXECUTION_BINDINGS_SCHEMA_V1: &str = "bijux.runner.execution_bindings.v1";
pub const RUNNER_EXECUTION_BINDINGS_SCHEMA_V2: &str = "bijux.runner.execution_bindings.v2";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandExecutionBindingV2 {
    pub command: String,
    pub program: String,
    #[serde(default)]
    pub prefix_args: Vec<String>,
    #[serde(default)]
    pub bind_root_envs: Vec<String>,
    #[serde(default)]
    pub required_files: Vec<BoundFileIdentityV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionBindingManifestV2 {
    pub schema_version: String,
    pub bindings: Vec<CommandExecutionBindingV2>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestVersion {
    schema_version: String,
    bindings: serde_json::Value,
}

#[derive(Debug)]
struct LoadedExecutionBinding {
    command: String,
    program: String,
    prefix_args: Vec<String>,
    bind_root_envs: Vec<String>,
    required_files: Vec<BoundFileIdentityV1>,
}

#[derive(Debug)]
struct LoadedExecutionBindingManifest {
    bindings: Vec<LoadedExecutionBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[cfg(test)]
pub(super) fn resolve_command(
    command: &str,
    args: &[String],
    envs: Option<&BTreeMap<String, String>>,
) -> Result<ResolvedCommand> {
    let environment = envs.map(|values| {
        values.iter().map(|(key, value)| (OsString::from(key), OsString::from(value))).collect()
    });
    resolve_command_with_environment(command, args, environment.as_ref())
}

pub(super) fn resolve_command_with_environment(
    command: &str,
    args: &[String],
    envs: Option<&BTreeMap<OsString, OsString>>,
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
    if !binding.bind_root_envs.is_empty() {
        let insertion_index = bind_argument_insertion_index(binding)?;
        let mut bind_args = Vec::with_capacity(binding.bind_root_envs.len() * 2);
        for variable in &binding.bind_root_envs {
            bind_args.push("--bind".to_string());
            bind_args.push(resolve_bind_root(variable, envs)?);
        }
        bound_args.splice(insertion_index..insertion_index, bind_args);
    }
    bound_args.extend_from_slice(args);
    Ok(ResolvedCommand { program: binding.program.clone(), args: bound_args })
}

fn configured_manifest_path(
    envs: Option<&BTreeMap<OsString, OsString>>,
) -> Result<Option<PathBuf>> {
    let configured = envs
        .and_then(|values| values.get(OsStr::new(RUNNER_EXECUTION_BINDINGS_ENV)).cloned())
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

fn load_manifest(path: &Path) -> Result<LoadedExecutionBindingManifest> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect runner execution binding manifest {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("runner execution binding manifest must be a regular non-symlink file");
    }
    let bytes = std::fs::read(path)
        .with_context(|| format!("read runner execution binding manifest {}", path.display()))?;
    let version: ManifestVersion = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse runner execution binding manifest {}", path.display()))?;
    let _ = version.bindings;
    match version.schema_version.as_str() {
        RUNNER_EXECUTION_BINDINGS_SCHEMA_V1 => {
            let manifest: ExecutionBindingManifestV1 = serde_json::from_slice(&bytes)
                .with_context(|| format!("parse v1 runner execution bindings {}", path.display()))?;
            load_v1_manifest(manifest)
        }
        RUNNER_EXECUTION_BINDINGS_SCHEMA_V2 => {
            let manifest: ExecutionBindingManifestV2 = serde_json::from_slice(&bytes)
                .with_context(|| format!("parse v2 runner execution bindings {}", path.display()))?;
            load_v2_manifest(manifest)
        }
        schema => bail!(
            "unsupported runner execution binding schema {schema}; expected {RUNNER_EXECUTION_BINDINGS_SCHEMA_V1} or {RUNNER_EXECUTION_BINDINGS_SCHEMA_V2}"
        ),
    }
}

fn load_v1_manifest(
    manifest: ExecutionBindingManifestV1,
) -> Result<LoadedExecutionBindingManifest> {
    if manifest.schema_version != RUNNER_EXECUTION_BINDINGS_SCHEMA_V1 {
        bail!("v1 runner execution binding manifest has the wrong schema version");
    }
    let bindings = manifest
        .bindings
        .into_iter()
        .map(|binding| LoadedExecutionBinding {
            command: binding.command,
            program: binding.program,
            prefix_args: binding.prefix_args,
            bind_root_envs: Vec::new(),
            required_files: binding.required_files,
        })
        .collect();
    validate_manifest(LoadedExecutionBindingManifest { bindings })
}

fn load_v2_manifest(
    manifest: ExecutionBindingManifestV2,
) -> Result<LoadedExecutionBindingManifest> {
    if manifest.schema_version != RUNNER_EXECUTION_BINDINGS_SCHEMA_V2 {
        bail!("v2 runner execution binding manifest has the wrong schema version");
    }
    let bindings = manifest
        .bindings
        .into_iter()
        .map(|binding| LoadedExecutionBinding {
            command: binding.command,
            program: binding.program,
            prefix_args: binding.prefix_args,
            bind_root_envs: binding.bind_root_envs,
            required_files: binding.required_files,
        })
        .collect();
    validate_manifest(LoadedExecutionBindingManifest { bindings })
}

fn validate_manifest(
    manifest: LoadedExecutionBindingManifest,
) -> Result<LoadedExecutionBindingManifest> {
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
        validate_bind_root_envs(binding)?;
        validate_required_files(binding)?;
    }
    Ok(manifest)
}

fn validate_bind_root_envs(binding: &LoadedExecutionBinding) -> Result<()> {
    let mut variables = BTreeSet::new();
    let mut previous: Option<&str> = None;
    for variable in &binding.bind_root_envs {
        validate_environment_variable(variable)?;
        if !variables.insert(variable.as_str()) {
            bail!("runner execution binding repeats bind-root environment {variable}");
        }
        if previous.is_some_and(|previous| previous >= variable.as_str()) {
            bail!("runner execution binding bind-root environments must be sorted");
        }
        previous = Some(variable);
    }
    if !binding.bind_root_envs.is_empty() {
        bind_argument_insertion_index(binding)?;
    }
    Ok(())
}

fn validate_required_files(binding: &LoadedExecutionBinding) -> Result<()> {
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

fn verify_required_files(binding: &LoadedExecutionBinding) -> Result<()> {
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

fn bind_argument_insertion_index(binding: &LoadedExecutionBinding) -> Result<usize> {
    if binding.prefix_args.first().map(String::as_str) != Some("exec") {
        bail!("bind-root environments require an Apptainer exec argument prefix");
    }
    binding
        .required_files
        .iter()
        .filter_map(|identity| {
            binding.prefix_args.iter().position(|argument| argument == &identity.path)
        })
        .min()
        .context("bind-root environments require an artifact path in the argument prefix")
}

fn resolve_bind_root(
    variable: &str,
    envs: Option<&BTreeMap<OsString, OsString>>,
) -> Result<String> {
    let value = match envs.and_then(|values| values.get(OsStr::new(variable))) {
        Some(value) => value.to_str().map(str::to_owned),
        None => std::env::var(variable).ok(),
    }
    .with_context(|| format!("bind-root environment {variable} is not set"))?;
    require_nonempty(&value, &format!("bind-root environment {variable}"))?;
    if value.contains([':', ',']) {
        bail!("bind-root environment {variable} contains an Apptainer bind delimiter");
    }
    let path = Path::new(&value);
    if !path.is_absolute() || path.parent().is_none() {
        bail!("bind-root environment {variable} must name an absolute non-root path");
    }
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("inspect bind-root environment {variable} at {value}"))?;
    if !metadata.is_dir() {
        bail!("bind-root environment {variable} must name a directory");
    }
    Ok(value)
}

fn validate_environment_variable(value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        || value.as_bytes()[0].is_ascii_digit()
    {
        bail!("bind-root environment name is invalid: {value}");
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

    fn binding_v2(path: &Path, sha256: String, size_bytes: u64) -> ExecutionBindingManifestV2 {
        ExecutionBindingManifestV2 {
            schema_version: RUNNER_EXECUTION_BINDINGS_SCHEMA_V2.to_string(),
            bindings: vec![CommandExecutionBindingV2 {
                command: "bcftools".to_string(),
                program: "/usr/bin/apptainer".to_string(),
                prefix_args: vec![
                    "exec".to_string(),
                    "--cleanenv".to_string(),
                    path.display().to_string(),
                    "bcftools".to_string(),
                ],
                bind_root_envs: vec!["TMPDIR".to_string()],
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
    fn binding_v2_mounts_declared_runtime_root_before_the_image() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let artifact = temporary.path().join("bcftools.sif");
        let runtime_root = temporary.path().join("scheduler-scratch");
        std::fs::write(&artifact, b"admitted-image")?;
        std::fs::create_dir(&runtime_root)?;
        let sha256 = bijux_dna_infra::hash_file_sha256(&artifact)?;
        let manifest = binding_v2(&artifact, sha256, artifact.metadata()?.len());
        let manifest_path = temporary.path().join("execution-bindings-v2.json");
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
        let envs = BTreeMap::from([
            (RUNNER_EXECUTION_BINDINGS_ENV.to_string(), manifest_path.display().to_string()),
            ("TMPDIR".to_string(), runtime_root.display().to_string()),
        ]);

        let resolved = resolve_command(
            "bcftools",
            &["view".to_string(), runtime_root.join("sample.vcf.gz").display().to_string()],
            Some(&envs),
        )?;

        assert_eq!(resolved.program, "/usr/bin/apptainer");
        assert_eq!(
            resolved.args,
            vec![
                "exec",
                "--cleanenv",
                "--bind",
                runtime_root.to_str().unwrap_or_default(),
                artifact.to_str().unwrap_or_default(),
                "bcftools",
                "view",
                runtime_root.join("sample.vcf.gz").to_str().unwrap_or_default(),
            ]
        );
        Ok(())
    }

    #[test]
    fn binding_v2_rejects_missing_or_broad_runtime_roots() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let artifact = temporary.path().join("bcftools.sif");
        std::fs::write(&artifact, b"admitted-image")?;
        let sha256 = bijux_dna_infra::hash_file_sha256(&artifact)?;
        let mut manifest = binding_v2(&artifact, sha256, artifact.metadata()?.len());
        manifest.bindings[0].bind_root_envs = vec!["BIJUX_TEST_UNSET_RUNTIME_ROOT".to_string()];
        let manifest_path = temporary.path().join("execution-bindings-v2.json");
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
        let manifest_only = BTreeMap::from([(
            RUNNER_EXECUTION_BINDINGS_ENV.to_string(),
            manifest_path.display().to_string(),
        )]);

        let Err(missing) = resolve_command("bcftools", &[], Some(&manifest_only)) else {
            anyhow::bail!("missing runtime root was accepted");
        };
        assert!(missing
            .to_string()
            .contains("bind-root environment BIJUX_TEST_UNSET_RUNTIME_ROOT is not set"));

        let root = BTreeMap::from([
            (RUNNER_EXECUTION_BINDINGS_ENV.to_string(), manifest_path.display().to_string()),
            ("BIJUX_TEST_UNSET_RUNTIME_ROOT".to_string(), "/".to_string()),
        ]);
        let Err(broad) = resolve_command("bcftools", &[], Some(&root)) else {
            anyhow::bail!("root bind was accepted");
        };
        assert!(broad.to_string().contains("absolute non-root path"));
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

#[cfg(test)]
#[cfg(unix)]
mod native_environment_contracts {
    use super::*;
    use std::os::unix::ffi::OsStringExt as _;

    #[test]
    fn native_manifest_path_keeps_its_original_bytes() -> Result<()> {
        let root = bijux_dna_testkit::tempdir_for("native-binding-path");
        let path = root.path().join(OsString::from_vec(b"manifest-\xff.json".to_vec()));
        let environment = BTreeMap::from([(
            OsString::from(RUNNER_EXECUTION_BINDINGS_ENV),
            path.as_os_str().to_os_string(),
        )]);
        assert_eq!(configured_manifest_path(Some(&environment))?, Some(path));
        Ok(())
    }

    #[test]
    fn owned_nontext_bind_root_keeps_the_existing_missing_text_refusal() {
        let environment = BTreeMap::from([(
            OsString::from("HOME"),
            OsString::from_vec(b"owned-home-\xff".to_vec()),
        )]);
        let Err(error) = resolve_bind_root("HOME", Some(&environment)) else {
            panic!("an owned non-text bind root must not expose an ambient HOME");
        };
        assert!(error.to_string().contains("bind-root environment HOME is not set"));
    }
}
