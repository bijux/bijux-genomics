use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::env::{
    available_runners, cache_dir, docker_image_exists, load_image_catalog, load_platform,
    resolve_image, run_shell_capture, PlatformSpec, RuntimeKind, ToolImageSpec,
};
use bijux_dna_api::v1::api::run::run_command;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// # Errors
/// Returns an error if image resolution fails.
pub fn print_env_images<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) -> Result<()> {
    let mut entries: Vec<_> = catalog.iter().collect();
    entries.sort_by_key(|(name, _)| *name);
    for (name, spec) in entries {
        let resolved = resolve_image(spec, platform)?;
        let digest = spec.digest.as_deref().unwrap_or("no digest");
        println!("{name}: {} ({digest})", resolved.full_name);
    }
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read.
pub fn print_env_registry_list(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    println!("tool\thas_docker\thas_apptainer\thas_smoke\tpinned");
    for row in parse_tools_registry_rows(&raw)? {
        let has_docker = row.runtimes.iter().any(|v| v == "docker") && row.dockerfile.is_some();
        let has_apptainer =
            row.runtimes.iter().any(|v| v == "apptainer") && row.apptainer_def.is_some();
        let has_smoke = row.version_cmd.is_some();
        let pinned = row
            .pinned_commit
            .as_deref()
            .is_some_and(|s| s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()));
        println!("{}\t{has_docker}\t{has_apptainer}\t{has_smoke}\t{pinned}", row.id);
    }
    Ok(())
}

/// # Errors
/// Returns an error if smoke script execution fails.
pub fn run_env_smoke(runtime: &str, tool: &str) -> Result<()> {
    let tools = selected_tools(tool);
    let registry_path = current_registry_path()?;
    if runtime == "apptainer" {
        ensure_apptainer_tools(
            &registry_path,
            &resolved_apptainer_hpc_root()?,
            &tools,
            true,
            false,
        )?;
        return Ok(());
    }
    run_env_with_tools(&registry_path, runtime, &tools, "contract")
}

fn normalize_stage_id(stage: &str) -> String {
    if stage.contains('.') {
        stage.to_string()
    } else {
        format!("fastq.{stage}")
    }
}

fn parse_registry(path: &Path) -> Result<String> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(raw)
}

/// # Errors
/// Returns an error if registry cannot be parsed.
pub fn registry_tools_for_stage(
    registry_path: &Path,
    stage: &str,
    scenario: Option<&str>,
    kind: &str,
) -> Result<Vec<String>> {
    let parsed = parse_registry(registry_path)?;
    let stage_id = normalize_stage_id(stage);
    let Some(stage_entry) =
        parse_stage_registry_rows(&parsed)?.into_iter().find(|entry| entry.id == stage_id)
    else {
        return Err(anyhow!("stage not found in registry: {stage_id}"));
    };

    let mut result = match kind {
        "all" => {
            let mut all = Vec::new();
            all.extend(stage_entry.primary_tools);
            all.extend(stage_entry.optional_alternatives);
            all.extend(stage_entry.validation_tools);
            all.extend(stage_entry.reporting_tools);
            all
        }
        "primary" => stage_entry.primary_tools,
        "optional" => stage_entry.optional_alternatives,
        "validation" => stage_entry.validation_tools,
        "reporting" => stage_entry.reporting_tools,
        "benchmark" => benchmark_tools_for_stage(&stage_id, scenario)?,
        other => {
            return Err(anyhow!(
                "unsupported registry tool kind `{other}`; expected one of all, primary, optional, validation, reporting, benchmark"
            ))
        }
    };
    result.sort();
    result.dedup();
    Ok(result)
}

fn benchmark_tools_for_stage(stage_id: &str, scenario: Option<&str>) -> Result<Vec<String>> {
    bijux_dna_api::v1::api::fastq::benchmark_tools_for_stage(stage_id, scenario)
}

/// # Errors
/// Returns an error if stage cannot be resolved.
pub fn run_env_smoke_for_stage(registry_path: &Path, runtime: &str, stage: &str) -> Result<()> {
    let tools = registry_tools_for_stage(registry_path, stage, None, "all")?;
    if tools.is_empty() {
        return Err(anyhow!("no tools found for stage {stage}"));
    }
    if runtime == "apptainer" {
        ensure_apptainer_tools(
            registry_path,
            &resolved_apptainer_hpc_root()?,
            &tools,
            true,
            false,
        )?;
        return Ok(());
    }
    run_env_with_tools(registry_path, runtime, &tools, "contract")
}

/// # Errors
/// Returns an error if prep script execution fails.
pub fn run_env_prep(
    registry_path: &Path,
    runtime: &str,
    tool: Option<&str>,
    stage: Option<&str>,
) -> Result<()> {
    if let Some(tool) = tool {
        let tools = selected_tools(tool);
        if runtime == "apptainer" {
            ensure_apptainer_tools(
                registry_path,
                &resolved_apptainer_hpc_root()?,
                &tools,
                false,
                false,
            )?;
            return Ok(());
        }
        return run_env_with_tools(registry_path, runtime, &tools, "version");
    }
    if let Some(stage) = stage {
        let tools = registry_tools_for_stage(registry_path, stage, None, "all")?;
        if tools.is_empty() {
            return Err(anyhow!("no tools found for stage {stage}"));
        }
        if runtime == "apptainer" {
            ensure_apptainer_tools(
                registry_path,
                &resolved_apptainer_hpc_root()?,
                &tools,
                false,
                false,
            )?;
            return Ok(());
        }
        return run_env_with_tools(registry_path, runtime, &tools, "version");
    }
    run_env_with_tools(registry_path, runtime, &[], "version")
}

fn run_env_with_tools(
    registry_path: &Path,
    runtime: &str,
    tools: &[String],
    smoke_level: &str,
) -> Result<()> {
    if matches!(runtime, "docker-arm64" | "docker-amd64") {
        return run_docker_env_with_tools(registry_path, runtime, tools, smoke_level);
    }
    Err(anyhow!(
        "unsupported runtime `{runtime}`; expected docker-arm64 | docker-amd64 | apptainer"
    ))
}

fn selected_tools(value: &str) -> Vec<String> {
    let tools = value
        .split(',')
        .map(str::trim)
        .filter(|tool| !tool.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tools.is_empty() {
        vec![value.trim().to_string()]
    } else {
        tools
    }
}

pub(crate) fn current_registry_path() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("resolve current working directory")?;
    if let Ok(configured) = std::env::var("BIJUX_TOOL_REGISTRY_PATH") {
        let trimmed = configured.trim();
        if !trimmed.is_empty() {
            let configured_path = PathBuf::from(trimmed);
            return Ok(if configured_path.is_absolute() {
                configured_path
            } else {
                cwd.join(configured_path)
            });
        }
    }
    Ok(bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml"))
}

fn run_docker_env_with_tools(
    registry_path: &Path,
    runtime: &str,
    tools: &[String],
    smoke_level: &str,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current working directory")?;
    let platform = docker_platform_for_runtime(runtime)?;
    let catalog = load_image_catalog().context("load docker image catalog")?;
    let raw = parse_registry(registry_path)?;
    let mut registry_rows = parse_tools_registry_rows(&raw)?
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect::<HashMap<_, _>>();
    let requested = if tools.is_empty() {
        registry_rows
            .values()
            .filter(|row| {
                row.runtimes.iter().any(|item| item == "docker")
                    && row.dockerfile.as_deref().is_some_and(|value| !value.trim().is_empty())
                    && !matches!(row.status.as_str(), "planned" | "experimental" | "dropped")
            })
            .map(|row| row.id.clone())
            .collect::<Vec<_>>()
    } else {
        tools.to_vec()
    };
    if requested.is_empty() {
        return Err(anyhow!("no docker-backed tools selected for runtime {runtime}"));
    }

    for tool_id in requested {
        let row = registry_rows
            .remove(&tool_id)
            .ok_or_else(|| anyhow!("tool not found in registry: {tool_id}"))?;
        let image_name = ensure_local_docker_image(&repo_root, runtime, &platform, &catalog, &row)?;
        for probe in docker_probe_commands(&row, smoke_level) {
            run_docker_probe(&image_name, &probe, row.expected_bin.as_deref())
                .with_context(|| format!("docker smoke {runtime} {tool_id}: `{probe}`"))?;
        }
    }
    Ok(())
}

fn docker_platform_for_runtime(runtime: &str) -> Result<PlatformSpec> {
    let platform = load_platform(None).context("load default docker platform")?;
    if platform.runner != RuntimeKind::Docker {
        return Err(anyhow!(
            "default platform must resolve to docker for runtime {runtime}, got {}",
            platform.runner
        ));
    }
    match runtime {
        "docker-arm64" if platform.arch == "arm64" => Ok(platform),
        "docker-amd64" if platform.arch == "amd64" => Ok(platform),
        "docker-arm64" | "docker-amd64" => Err(anyhow!(
            "runtime {runtime} is unsupported on configured docker platform arch `{}`",
            platform.arch
        )),
        other => Err(anyhow!("unsupported docker runtime `{other}`")),
    }
}

fn ensure_local_docker_image<S: ::std::hash::BuildHasher>(
    repo_root: &Path,
    runtime: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec, S>,
    row: &RegistryRow,
) -> Result<String> {
    let dockerfile_rel = row
        .dockerfile
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("tool `{}` is missing dockerfile", row.id))?;
    let dockerfile_path = repo_root.join(dockerfile_rel);
    if !dockerfile_path.is_file() {
        return Err(anyhow!("missing dockerfile {}", dockerfile_path.display()));
    }
    let container_dir = repo_root.join(&platform.container_dir);
    if !container_dir.is_dir() {
        return Err(anyhow!("missing docker container dir {}", container_dir.display()));
    }
    let spec = catalog
        .get(&row.id)
        .ok_or_else(|| anyhow!("missing image catalog entry for {}", row.id))?;
    let local_image_name = buildable_docker_image_name(&row.id, &spec.version, platform)?;
    let git_revision = capture_optional_shell("git rev-parse HEAD");
    let dockerfile_sha256 = dockerfile_sha256(&dockerfile_path)?;
    let build_state_path = docker_build_state_path(repo_root, runtime, &row.id);
    if local_docker_image_exists(&local_image_name)
        && docker_build_state_matches(
            &build_state_path,
            &local_image_name,
            &git_revision,
            &dockerfile_sha256,
        )
    {
        return Ok(local_image_name);
    }

    let oci_created = capture_optional_shell("date -u +%Y-%m-%dT%H:%M:%SZ");
    let version = spec.version.trim();
    let output = run_command(
        "docker",
        &[
            "build".to_string(),
            "-t".to_string(),
            local_image_name.clone(),
            "--build-arg".to_string(),
            format!("OCI_REVISION={git_revision}"),
            "--build-arg".to_string(),
            format!("OCI_CREATED={oci_created}"),
            "--build-arg".to_string(),
            format!("TOOL_VERSION={version}"),
            "-f".to_string(),
            dockerfile_path.display().to_string(),
            container_dir.display().to_string(),
        ],
    )
    .with_context(|| format!("docker build {}", local_image_name))?;
    if output.exit_code == 0 {
        if let Some(parent) = build_state_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let state = DockerBuildState {
            image_name: local_image_name.clone(),
            git_revision,
            dockerfile_sha256,
        };
        bijux_dna_infra::atomic_write_json(&build_state_path, &state)
            .with_context(|| format!("write {}", build_state_path.display()))?;
        return Ok(local_image_name);
    }
    Err(anyhow!(
        "docker build failed for {}:\n{}",
        local_image_name,
        merge_command_output(&output.stdout, &output.stderr)
    ))
}

fn buildable_docker_image_name(
    tool_id: &str,
    version: &str,
    platform: &PlatformSpec,
) -> Result<String> {
    let version = version.trim();
    if version.is_empty() {
        return Err(anyhow!("tool `{tool_id}` is missing docker image version"));
    }
    Ok(format!("{}/{}:{}-{}", platform.image_prefix, tool_id, version, platform.arch))
}

fn docker_build_state_path(repo_root: &Path, runtime: &str, tool_id: &str) -> PathBuf {
    repo_root.join("artifacts/containers").join(runtime).join(format!("{tool_id}.build.json"))
}

fn local_docker_image_exists(image_name: &str) -> bool {
    run_command("docker", &["image".to_string(), "inspect".to_string(), image_name.to_string()])
        .is_ok_and(|output| output.exit_code == 0)
}

fn docker_build_state_matches(
    path: &Path,
    image_name: &str,
    git_revision: &str,
    dockerfile_sha256: &str,
) -> bool {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(state) = serde_json::from_str::<DockerBuildState>(&raw) else {
        return false;
    };
    state.image_name == image_name
        && state.git_revision == git_revision
        && state.dockerfile_sha256 == dockerfile_sha256
}

fn dockerfile_sha256(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(sha256_hex(&Sha256::digest(&bytes)))
}

fn capture_optional_shell(command: &str) -> String {
    run_shell_capture(command)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn docker_probe_commands(row: &RegistryRow, smoke_level: &str) -> Vec<String> {
    let version = resolved_probe_command(
        row.smoke_version_cmd.as_deref(),
        row.version_cmd.as_deref(),
        &format!("{} --version", row.id),
    );
    if smoke_level == "version" {
        return vec![version];
    }
    if !row.smoke_probes.is_empty() {
        return row.smoke_probes.clone();
    }
    let mut probes = vec![version];
    if row.smoke_require_help.unwrap_or(true) {
        probes.push(resolved_probe_command(
            row.smoke_help_cmd.as_deref(),
            row.help_cmd.as_deref(),
            &format!("{} --help", row.id),
        ));
    }
    probes
}

fn resolved_probe_command(primary: Option<&str>, fallback: Option<&str>, default: &str) -> String {
    primary
        .or(fallback)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default.to_string())
}

fn run_docker_probe(image_name: &str, probe: &str, expected_bin: Option<&str>) -> Result<String> {
    let output = match docker_probe_invocation(probe, expected_bin) {
        DockerProbeInvocation::EntrypointArgs(args) => {
            let mut argv = vec![
                "run".to_string(),
                "--rm".to_string(),
                "--network".to_string(),
                "none".to_string(),
                image_name.to_string(),
            ];
            argv.extend(args);
            run_command("docker", &argv).with_context(|| format!("docker run {}", image_name))?
        }
        DockerProbeInvocation::Shell(command) => run_command(
            "docker",
            &[
                "run".to_string(),
                "--rm".to_string(),
                "--network".to_string(),
                "none".to_string(),
                "--entrypoint".to_string(),
                "/bin/sh".to_string(),
                image_name.to_string(),
                "-lc".to_string(),
                command,
            ],
        )
        .with_context(|| format!("docker run {}", image_name))?,
    };
    let merged = merge_command_output(&output.stdout, &output.stderr);
    if output.exit_code == 0 {
        return Ok(merged);
    }
    Err(anyhow!(merged))
}

#[derive(Debug, PartialEq, Eq)]
enum DockerProbeInvocation {
    EntrypointArgs(Vec<String>),
    Shell(String),
}

fn docker_probe_invocation(probe: &str, expected_bin: Option<&str>) -> DockerProbeInvocation {
    let trimmed = probe.trim();
    let Some(expected_bin) = expected_bin.map(str::trim).filter(|value| !value.is_empty()) else {
        return DockerProbeInvocation::Shell(trimmed.to_string());
    };
    let mut tokens = trimmed.split_whitespace().map(ToOwned::to_owned).collect::<Vec<_>>();
    if tokens.first().is_some_and(|token| token == expected_bin) {
        tokens.remove(0);
        DockerProbeInvocation::EntrypointArgs(tokens)
    } else {
        DockerProbeInvocation::Shell(trimmed.to_string())
    }
}

fn merge_command_output(stdout: &str, stderr: &str) -> String {
    let stdout = stdout.trim().to_string();
    let stderr = stderr.trim().to_string();
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("{stdout}\n{stderr}"),
    }
}

fn resolved_apptainer_hpc_root() -> Result<PathBuf> {
    if let Ok(config) = crate::commands::hpc::load_hpc_config() {
        return Ok(config.resolve_paths().root);
    }
    Err(anyhow!("unable to resolve Apptainer HPC root: declare BIJUX_HPC_CONFIG"))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod env_registry_query_tests {
    use super::{
        current_registry_path, docker_probe_commands, docker_probe_invocation,
        parse_tools_registry_rows, registry_tools_for_stage, selected_tools, DockerProbeInvocation,
        RegistryRow,
    };

    fn registry_path() -> std::path::PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root()
            .expect("repo root")
            .join("configs/ci/registry/tool_registry.toml")
    }

    #[test]
    fn registry_benchmark_tools_follow_trim_fairness_cohort() {
        let tools =
            registry_tools_for_stage(&registry_path(), "fastq.trim_reads", None, "benchmark")
                .expect("trim benchmark tools");
        assert_eq!(
            tools,
            vec![
                "adapterremoval",
                "alientrimmer",
                "atropos",
                "bbduk",
                "cutadapt",
                "fastp",
                "fastx_clipper",
                "leehom",
                "prinseq",
                "seqkit",
                "skewer",
                "trim_galore",
                "trimmomatic",
            ]
        );
    }

    #[test]
    fn registry_benchmark_tools_follow_polyg_fairness_cohort() {
        let tools =
            registry_tools_for_stage(&registry_path(), "fastq.trim_polyg_tails", None, "benchmark")
                .expect("polyg benchmark tools");
        assert_eq!(tools, vec!["bbduk", "fastp"]);
    }

    #[test]
    fn registry_rejects_unknown_tool_kind() {
        let error = registry_tools_for_stage(&registry_path(), "fastq.trim_reads", None, "bogus")
            .expect_err("unknown kind should fail");
        assert!(error.to_string().contains("unsupported registry tool kind"));
    }

    #[test]
    fn registry_benchmark_tools_follow_explicit_profile_read_length_cohort() {
        let tools = registry_tools_for_stage(
            &registry_path(),
            "fastq.profile_read_lengths",
            Some("read_length_fairness"),
            "benchmark",
        )
        .expect("read-length benchmark tools");
        assert_eq!(tools, vec!["fastp", "prinseq", "seqfu", "seqkit_stats"]);
    }

    #[test]
    fn selected_tools_supports_csv_batches() {
        assert_eq!(selected_tools("fastp, seqkit ,cutadapt"), vec!["fastp", "seqkit", "cutadapt"]);
    }

    #[test]
    fn selected_tools_preserves_single_tool_input() {
        assert_eq!(selected_tools("fastp"), vec!["fastp"]);
    }

    #[test]
    fn docker_probe_invocation_uses_image_entrypoint_when_probe_matches_expected_bin() {
        let invocation = docker_probe_invocation("adapterremoval --help", Some("adapterremoval"));
        assert_eq!(invocation, DockerProbeInvocation::EntrypointArgs(vec!["--help".to_string()]));
    }

    #[test]
    fn docker_probe_invocation_falls_back_to_shell_when_probe_uses_wrapper_command() {
        let invocation =
            docker_probe_invocation("sh -lc 'adapterremoval --help'", Some("adapterremoval"));
        assert_eq!(
            invocation,
            DockerProbeInvocation::Shell("sh -lc 'adapterremoval --help'".to_string())
        );
    }

    #[test]
    fn contract_probe_commands_default_to_version_and_help() {
        let row = RegistryRow {
            id: "adapterremoval".to_string(),
            version_cmd: Some("adapterremoval --version".to_string()),
            help_cmd: Some("adapterremoval --help".to_string()),
            smoke_require_help: Some(true),
            ..RegistryRow::default()
        };
        assert_eq!(
            docker_probe_commands(&row, "contract"),
            vec!["adapterremoval --version".to_string(), "adapterremoval --help".to_string()]
        );
    }

    #[test]
    fn parse_tools_registry_rows_keeps_last_tool_before_stage_section() {
        let rows = parse_tools_registry_rows(
            r#"
[[tools]]
id = "vsearch"
runtimes = ["docker"]

[[tools]]
id = "yleaf"
runtimes = ["docker", "apptainer"]

[[stages]]
id = "bam.haplogroups"
"#,
        )
        .expect("parse test registry");
        assert_eq!(
            rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
            vec!["vsearch", "yleaf"]
        );
    }

    #[test]
    fn current_registry_path_honors_env_variable() {
        let original = std::env::var_os("BIJUX_TOOL_REGISTRY_PATH");
        std::env::set_var("BIJUX_TOOL_REGISTRY_PATH", "configs/ci/registry/tool_registry_vcf.toml");
        let path = current_registry_path().expect("resolve registry path from env variable");
        if let Some(value) = original {
            std::env::set_var("BIJUX_TOOL_REGISTRY_PATH", value);
        } else {
            std::env::remove_var("BIJUX_TOOL_REGISTRY_PATH");
        }
        assert!(path.ends_with("configs/ci/registry/tool_registry_vcf.toml"));
    }
}

#[derive(Default, Serialize)]
struct RegistryRow {
    id: String,
    status: String,
    domain: Option<String>,
    domains: Vec<String>,
    stage_ids: Vec<String>,
    bindings: Vec<String>,
    tool_role: Option<String>,
    version: Option<String>,
    upstream: Option<String>,
    runtimes: Vec<String>,
    dockerfile: Option<String>,
    apptainer_def: Option<String>,
    version_cmd: Option<String>,
    help_cmd: Option<String>,
    expected_bin: Option<String>,
    pinned_commit: Option<String>,
    container_ref: Option<String>,
    expected_version_regex: Option<String>,
    healthcheck_cmd: Option<String>,
    smoke_version_cmd: Option<String>,
    smoke_help_cmd: Option<String>,
    smoke_require_help: Option<bool>,
    smoke_probes: Vec<String>,
    java_heap_mb: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DockerBuildState {
    image_name: String,
    git_revision: String,
    dockerfile_sha256: String,
}

#[derive(Debug, Serialize)]
pub struct EnsureImagesReport {
    pub schema_version: &'static str,
    pub domain: String,
    pub stages: Vec<String>,
    pub tools: Vec<EnsureToolReport>,
    pub built: usize,
    pub reused: usize,
    pub quick_smoked: usize,
    pub failed: usize,
}

#[derive(Debug, Serialize)]
pub struct EnsureToolReport {
    pub tool_id: String,
    pub stage_id: String,
    pub sif_path: String,
    pub registry_digest: String,
    pub sif_sha256: String,
    pub built: bool,
    pub quick_smoked: bool,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SifInventoryReport {
    pub schema_version: &'static str,
    pub containers_dir: String,
    pub entries: Vec<SifInventoryEntry>,
}

#[derive(Debug, Serialize)]
pub struct SifInventoryEntry {
    pub tool_id: String,
    pub sif_path: String,
    pub sha256: String,
    pub smoke_manifest_path: Option<String>,
    pub smoke_status: Option<String>,
}

#[derive(Debug, Serialize)]
struct SmokeProbeResult {
    command: String,
    applied_command: String,
    ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    output_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    output_first_line: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct SmokeManifest {
    schema_version: &'static str,
    tool_id: String,
    stage_id: String,
    status: String,
    registry_digest: String,
    sif_sha256: String,
    version_cmd: String,
    help_cmd: String,
    version: String,
    version_output_first_line: String,
    help_ok: bool,
    quick_smoke: bool,
    probe_commands: Vec<String>,
    #[serde(default)]
    probe_results: Vec<SmokeProbeResult>,
    java_heap_mb: Option<u64>,
    upstream: String,
    image_build_timestamp_unix_s: u64,
    checked_at_unix_s: u64,
}

fn parse_tools_registry_rows(raw: &str) -> Result<Vec<RegistryRow>> {
    let mut rows = Vec::new();
    let mut current: Option<RegistryRow> = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
            if let Some(row) = current.take() {
                rows.push(row);
            }
            if trimmed == "[[tools]]" {
                current = Some(RegistryRow::default());
            }
            continue;
        }
        let Some(tool_entry) = current.as_mut() else {
            continue;
        };
        if let Some(value) = parse_toml_string(trimmed, "id") {
            tool_entry.id = value;
        } else if let Some(value) = parse_toml_string(trimmed, "status") {
            tool_entry.status = value;
        } else if let Some(value) = parse_toml_string(trimmed, "domain") {
            tool_entry.domain = Some(value);
        } else if let Some(values) = parse_toml_array(trimmed, "domains") {
            tool_entry.domains = values;
        } else if let Some(values) = parse_toml_array(trimmed, "stage_ids") {
            tool_entry.stage_ids = values;
        } else if let Some(values) = parse_toml_array(trimmed, "bindings") {
            tool_entry.bindings = values;
        } else if let Some(value) = parse_toml_string(trimmed, "tool_role") {
            tool_entry.tool_role = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "version") {
            tool_entry.version = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "upstream") {
            tool_entry.upstream = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "dockerfile") {
            tool_entry.dockerfile = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "apptainer_def") {
            tool_entry.apptainer_def = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "version_cmd") {
            tool_entry.version_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "help_cmd") {
            tool_entry.help_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "expected_bin") {
            tool_entry.expected_bin = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "pinned_commit") {
            tool_entry.pinned_commit = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "container_ref") {
            tool_entry.container_ref = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "expected_version_regex") {
            tool_entry.expected_version_regex = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "healthcheck_cmd") {
            tool_entry.healthcheck_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "smoke_version_cmd") {
            tool_entry.smoke_version_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "smoke_help_cmd") {
            tool_entry.smoke_help_cmd = Some(value);
        } else if let Some(value) = parse_toml_bool(trimmed, "smoke_require_help") {
            tool_entry.smoke_require_help = Some(value);
        } else if let Some(values) = parse_toml_array(trimmed, "smoke_probes") {
            tool_entry.smoke_probes = values;
        } else if let Some(value) = parse_toml_u64(trimmed, "java_heap_mb") {
            tool_entry.java_heap_mb = Some(value);
        } else if let Some(values) = parse_toml_array(trimmed, "runtimes") {
            tool_entry.runtimes = values;
        }
    }
    if let Some(row) = current {
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(anyhow!("missing [[tools]] entries"));
    }
    Ok(rows)
}
